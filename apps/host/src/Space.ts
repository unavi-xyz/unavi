import { ResponseMessage } from "@wired-protocol/types";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";

import { Player } from "./Player";
import { SpaceRegistry } from "./SpaceRegistry";
import { toHex } from "./utils/toHex";

export class Space {
  readonly uri: string;
  #registry: SpaceRegistry;

  players = new Map<number, Player>();

  constructor(uri: string, registry: SpaceRegistry) {
    this.uri = uri;
    this.#registry = registry;
  }

  get topic() {
    return `space/${this.uri}`;
  }

  get playerCount() {
    return this.players.size;
  }

  playerId(player: Player) {
    return Array.from(this.players.keys()).find((id) => this.players.get(id) === player);
  }

  join(player: Player) {
    // Find an open player id
    const playerIds = Array.from(this.players.keys());
    const triedPlayerIds: number[] = [];
    let i = 0;
    let playerId: number | null = null;

    while (playerId === null) {
      // Pick a random id
      let id: number;
      while (triedPlayerIds.includes((id = Math.floor(Math.random() * 256))));
      triedPlayerIds.push(id);

      // If it's not taken, use it
      if (!playerIds.includes(id)) playerId = id;

      i++;
      if (i === 256) {
        console.warn("❌ No open player ids.");
        player.leave(this.uri);
        return;
      }
    }

    this.players.set(playerId, player);

    this.#publish({
      type: "player_join",
      data: {
        playerId,
        name: player.name || undefined,
        avatar: player.avatar || undefined,
        address: player.address || undefined,
      },
    });

    // Tell new player about current players
    this.players.forEach((otherPlayer, otherPlayerId) => {
      if (otherPlayer === player) return;

      player.send({
        type: "player_join",
        data: {
          playerId: otherPlayerId,
          name: otherPlayer.name || undefined,
          avatar: otherPlayer.avatar || undefined,
          address: otherPlayer.address || undefined,
        },
      });

      // Consume current players
      if (otherPlayer.producer) player.consume(otherPlayer.producer, this.uri, otherPlayerId);
      if (otherPlayer.dataProducer)
        player.consumeData(otherPlayer.dataProducer, this.uri, otherPlayerId);
    });

    // Start producing
    if (player.producer) this.setProducer(player, player.producer);
    if (player.dataProducer) this.setDataProducer(player, player.dataProducer);

    console.info(`👋 Player ${toHex(playerId)} joined space ${this.uri}.`);
  }

  leave(player: Player) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.players.forEach((otherPlayer) => {
      if (otherPlayer === player) return;

      const consumers = player.consumers.get(this);
      if (consumers) {
        const consumer = consumers.get(playerId);
        if (consumer) consumer.close();
      }

      const dataConsumers = player.dataConsumers.get(this);
      if (dataConsumers) {
        const dataConsumer = dataConsumers.get(playerId);
        if (dataConsumer) dataConsumer.close();
      }
    });

    this.players.delete(playerId);

    this.#publish({ type: "player_leave", data: playerId });

    console.info(`👋 Player ${toHex(playerId)} left space ${this.uri}.`);

    if (this.playerCount === 0) this.#registry.removeSpace(this.uri);
  }

  chat(player: Player, message: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ type: "chat_message", data: { playerId, message } });
  }

  setGrounded(player: Player, grounded: boolean) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ type: "player_grounded", data: { playerId, grounded } });
  }

  setName(player: Player, name: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      type: "player_name",
      data: { playerId, name: name },
    });
  }

  setAddress(player: Player, address: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ type: "player_address", data: { playerId, address } });
  }

  setAvatar(player: Player, avatar: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      type: "player_avatar",
      data: { playerId, avatar: avatar },
    });
  }

  setProducer(player: Player, producer: Producer) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    // Create a consumer for each player
    this.players.forEach((otherPlayer) => {
      if (otherPlayer === player) return;
      otherPlayer.consume(producer, this.uri, playerId);
    });
  }

  setDataProducer(player: Player, dataProducer: DataProducer) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    // Create a data consumer for each player
    this.players.forEach((otherPlayer) => {
      if (otherPlayer === player) return;
      otherPlayer.consumeData(dataProducer, this.uri, playerId);
    });
  }

  #publish(message: ResponseMessage) {
    this.#registry.server.publish(this.topic, JSON.stringify(message));
  }
}
