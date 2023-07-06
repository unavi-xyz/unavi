import { ResponseMessage } from "@wired-protocol/types";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";

import { Player } from "./Player";
import { toHex } from "./utils/toHex";
import { WorldRegistry } from "./WorldRegistry";

export class World {
  readonly uri: string;
  #registry: WorldRegistry;

  players = new Map<number, Player>();

  constructor(uri: string, registry: WorldRegistry) {
    this.uri = uri;
    this.#registry = registry;
  }

  get topic() {
    return `world/${this.uri}`;
  }

  get playerCount() {
    return this.players.size;
  }

  playerId(player: Player) {
    return Array.from(this.players.keys()).find(
      (id) => this.players.get(id) === player
    );
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
      data: {
        avatar: player.avatar || undefined,
        handle: player.handle || undefined,
        name: player.name || undefined,
        playerId,
      },
      id: "com.wired-protocol.world.player.join",
    });

    // Tell new player about current players
    this.players.forEach((otherPlayer, otherPlayerId) => {
      if (otherPlayer === player) return;

      player.send({
        data: {
          avatar: otherPlayer.avatar || undefined,
          handle: otherPlayer.handle || undefined,
          name: otherPlayer.name || undefined,
          playerId: otherPlayerId,
        },
        id: "com.wired-protocol.world.player.join",
      });

      // Consume current players
      if (otherPlayer.producer)
        player.consume(otherPlayer.producer, this.uri, otherPlayerId);
      if (otherPlayer.dataProducer)
        player.consumeData(otherPlayer.dataProducer, this.uri, otherPlayerId);
    });

    // Start producing
    if (player.producer) this.setProducer(player, player.producer);
    if (player.dataProducer) this.setDataProducer(player, player.dataProducer);

    console.info(`👋 Player ${toHex(playerId)} joined world ${this.uri}`);
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

    this.#publish({
      data: playerId,
      id: "com.wired-protocol.world.player.leave",
    });

    console.info(`👋 Player ${toHex(playerId)} left world ${this.uri}`);

    if (this.playerCount === 0) this.#registry.removeWorld(this.uri);
  }

  chat(player: Player, message: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      data: { message, playerId },
      id: "com.wired-protocol.world.chat.message",
    });
  }

  setFalling(player: Player, falling: boolean) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      data: { falling, playerId },
      id: "com.wired-protocol.world.player.falling",
    });
  }

  setName(player: Player, name: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      data: { name: name, playerId },
      id: "com.wired-protocol.world.player.name",
    });
  }

  setHandle(player: Player, handle: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      data: { handle, playerId },
      id: "com.wired-protocol.world.player.handle",
    });
  }

  setAvatar(player: Player, avatar: string | null) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    this.#publish({
      data: { avatar: avatar, playerId },
      id: "com.wired-protocol.world.player.avatar",
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
