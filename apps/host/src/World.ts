import {
  ChatMessage,
  Message,
  PlayerAvatar,
  PlayerFalling,
  PlayerHandle,
  PlayerJoined,
  PlayerLeft,
  PlayerNickname,
} from "@wired-protocol/types";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";

import { RES_WORLD } from "./constants";
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
      data: PlayerJoined.toBinary(playerJoined),
      type: `${RES_WORLD}.PlayerJoined`,
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
        data: PlayerJoined.toBinary(otherPlayerJoined),
        type: `${RES_WORLD}.PlayerJoined`,
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
      data: PlayerLeft.toBinary(playerLeft),
      type: `${RES_WORLD}.PlayerLeft`,
    });

    console.info(`👋 Player ${toHex(playerId)} left world ${this.uri}`);

    if (this.playerCount === 0) this.#registry.removeWorld(this.uri);
  }

  chat(player: Player, message: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const chatMessage = ChatMessage.create({ message, playerId });

    this.#publish({
      data: ChatMessage.toBinary(chatMessage),
      type: `${RES_WORLD}.ChatMessage`,
    });
  }

  setFalling(player: Player, falling: boolean) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerFalling = PlayerFalling.create({ falling, playerId });

    this.#publish({
      data: PlayerFalling.toBinary(playerFalling),
      type: `${RES_WORLD}.PlayerFalling`,
    });
  }

  setName(player: Player, nickname: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerNickname = PlayerNickname.create({ nickname, playerId });

    this.#publish({
      data: PlayerNickname.toBinary(playerNickname),
      type: `${RES_WORLD}.PlayerNickname`,
    });
  }

  setHandle(player: Player, handle: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerHandle = PlayerHandle.create({ handle, playerId });

    this.#publish({
      data: PlayerHandle.toBinary(playerHandle),
      type: `${RES_WORLD}.PlayerHandle`,
    });
  }

  setAvatar(player: Player, avatar: string) {
    const playerId = this.playerId(player);
    if (playerId === undefined) return;

    const playerAvatar = PlayerAvatar.create({ avatar, playerId });

    this.#publish({
      data: PlayerAvatar.toBinary(playerAvatar),
      type: `${RES_WORLD}.PlayerAvatar`,
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

  #publish(data: Partial<Message>) {
    const message = Message.create(data);
    this.#registry.server.publish(this.topic, Message.toBinary(message), true);
  }
}
