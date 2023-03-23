import { FromHostMessage } from "@wired-labs/protocol";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";

import { Player } from "./Player";
import { SpaceRegistry } from "./SpaceRegistry";
import { toHex } from "./utils/toHex";

export class Space {
  readonly id: number;
  #registry: SpaceRegistry;

  players = new Map<number, Player>();

  constructor(id: number, registry: SpaceRegistry) {
    this.id = id;
    this.#registry = registry;
  }

  get topic() {
    return `space/${this.id}`;
  }

  get playercount() {
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
        player.leave(this.id);
        return;
      }
    }

    this.players.set(playerId, player);

    this.#publish({
      subject: "player_joined",
      data: { playerId, name: player.name, address: player.address, avatar: player.avatar },
    });

    // Tell new player about current players
    this.players.forEach((otherPlayer, otherPlayerId) => {
      if (otherPlayer === player) return;

      player.send({
        subject: "player_joined",
        data: {
          playerId: otherPlayerId,
          name: otherPlayer.name,
          address: otherPlayer.address,
          avatar: otherPlayer.avatar,
          beforeYou: true,
        },
      });

      // Consume current players
      if (otherPlayer.producer) player.consume(otherPlayer.producer, this.id, otherPlayerId);
      if (otherPlayer.dataProducer)
        player.consumeData(otherPlayer.dataProducer, this.id, otherPlayerId);
    });

    // Start producing
    if (player.producer) this.setProducer(player, player.producer);
    if (player.dataProducer) this.setDataProducer(player, player.dataProducer);

    console.info(`👋 Player ${toHex(playerId)} joined space ${toHex(this.id)}.`);
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

    this.#publish({ subject: "player_left", data: { playerId } });

    console.info(`👋 Player ${toHex(playerId)} left space ${toHex(this.id)}.`);

    if (this.playercount === 0) this.#registry.removeSpace(this.id);
  }

  chat(player: Player, text: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ subject: "player_chat", data: { playerId, text, timestamp: Date.now() } });
  }

  setGrounded(player: Player, grounded: boolean) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ subject: "player_grounded", data: { playerId, grounded } });
  }

  setName(player: Player, name: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ subject: "player_name", data: { playerId, name } });
  }

  setAddress(player: Player, address: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ subject: "player_address", data: { playerId, address } });
  }

  setAvatar(player: Player, avatar: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({ subject: "player_avatar", data: { playerId, avatar } });
  }

  setProducer(player: Player, producer: Producer) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    // Create a consumer for each player
    this.players.forEach((otherPlayer) => {
      if (otherPlayer === player) return;
      otherPlayer.consume(producer, this.id, playerId);
    });
  }

  setDataProducer(player: Player, dataProducer: DataProducer) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    // Create a data consumer for each player
    this.players.forEach((otherPlayer) => {
      if (otherPlayer === player) return;
      otherPlayer.consumeData(dataProducer, this.id, playerId);
    });
  }

  #publish(message: FromHostMessage) {
    this.#registry.server.publish(this.topic, JSON.stringify(message));
  }
}
