import {
  ChatMessage,
  PlayerAvatar,
  PlayerFalling,
  PlayerHandle,
  PlayerJoined,
  PlayerLeft,
  PlayerNickname,
  Response,
} from "@wired-protocol/types";
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
      (id) => this.players.get(id) === player,
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

    const playerJoined = PlayerJoined.create({
      avatar: player.avatar || undefined,
      handle: player.handle || undefined,
      nickname: player.name || undefined,
      playerId,
    });

    this.#publish({
      oneofKind: "playerJoined",
      playerJoined,
    });

    // Tell new player about current players
    this.players.forEach((otherPlayer, otherPlayerId) => {
      if (otherPlayer === player) return;

      const otherPlayerJoined = PlayerJoined.create({
        avatar: otherPlayer.avatar || undefined,
        handle: otherPlayer.handle || undefined,
        nickname: otherPlayer.name || undefined,
        playerId: otherPlayerId,
      });

      player.send({
        oneofKind: "playerJoined",
        playerJoined: otherPlayerJoined,
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

    const playerLeft = PlayerLeft.create({ playerId });

    this.#publish({
      oneofKind: "playerLeft",
      playerLeft,
    });

    console.info(`👋 Player ${toHex(playerId)} left world ${this.uri}`);

    if (this.playerCount === 0) this.#registry.removeWorld(this.uri);
  }

  chat(player: Player, message: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const chatMessage = ChatMessage.create({ message, playerId });

    this.#publish({
      chatMessage,
      oneofKind: "chatMessage",
    });
  }

  setFalling(player: Player, falling: boolean) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerFalling = PlayerFalling.create({ falling, playerId });

    this.#publish({
      oneofKind: "playerFalling",
      playerFalling,
    });
  }

  setName(player: Player, nickname: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerNickname = PlayerNickname.create({ nickname, playerId });

    this.#publish({
      oneofKind: "playerNickname",
      playerNickname,
    });
  }

  setHandle(player: Player, handle: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerHandle = PlayerHandle.create({ handle, playerId });

    this.#publish({
      oneofKind: "playerHandle",
      playerHandle,
    });
  }

  setAvatar(player: Player, avatar: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerAvatar = PlayerAvatar.create({ avatar, playerId });

    this.#publish({
      oneofKind: "playerAvatar",
      playerAvatar,
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

  #publish(data: Response["response"]) {
    const message = Response.create({ response: data });
    this.#registry.server.publish(this.topic, Response.toBinary(message), true);
  }
}
