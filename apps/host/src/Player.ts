import {
  CreateConsumer,
  CreateDataConsumer,
  DataProducerId,
  JoinSuccess,
  ProducerId,
  Response,
  TransportType,
} from "@wired-protocol/types";
import { Consumer } from "mediasoup/node/lib/Consumer";
import { DataConsumer } from "mediasoup/node/lib/DataConsumer";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";
import {
  RtpCapabilities,
  RtpParameters,
} from "mediasoup/node/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup/node/lib/SctpParameters";
import { WebRtcTransport } from "mediasoup/node/lib/types";

import { uWebSocket } from "./types";
import { World } from "./World";
import { WorldRegistry } from "./WorldRegistry";

export class Player {
  ws: uWebSocket | null = null;
  #registry: WorldRegistry;

  worlds = new Set<World>();
  consumers = new Map<World, Map<number, Consumer>>();
  dataConsumers = new Map<World, Map<number, DataConsumer>>();

  playerData: Record<string, string> = {};

  #producer: Producer | null = null;
  #dataProducer: DataProducer | null = null;
  #rtpCapabilities: RtpCapabilities | null = null;

  consumerTransport: WebRtcTransport | null = null;
  producerTransport: WebRtcTransport | null = null;

  constructor(ws: uWebSocket, registry: WorldRegistry) {
    this.ws = ws;
    this.#registry = registry;
  }

  send(data: Response["response"]) {
    const message = Response.create({ response: data });
    this.ws?.send(Response.toBinary(message), true);
  }

  get rtpCapabilities() {
    return this.#rtpCapabilities;
  }

  set rtpCapabilities(rtpCapabilities: RtpCapabilities | null) {
    this.#rtpCapabilities = rtpCapabilities;
    if (this.consumerTransport) this.#startConsuming();
  }

  get producer() {
    return this.#producer;
  }

  set producer(producer: Producer | null) {
    this.#producer = producer;
    if (producer)
      this.worlds.forEach((world) => world.setProducer(this, producer));
  }

  get dataProducer() {
    return this.#dataProducer;
  }

  set dataProducer(dataProducer: DataProducer | null) {
    this.#dataProducer = dataProducer;
    if (dataProducer)
      this.worlds.forEach((world) => world.setDataProducer(this, dataProducer));
  }

  setPlayerData(key: string, value: string) {
    this.playerData[key] = value;
    this.worlds.forEach((world) => world.setPlayerData(this, key, value));
  }

  join(uri: string) {
    const world = this.#registry.getOrCreateWorld(uri);
    if (this.worlds.has(world)) return;

    world.join(this);
    this.worlds.add(world);

    const playerId = world.playerId(this);
    if (playerId === undefined) return;

    this.ws?.subscribe(world.topic);

    const joinSuccess = JoinSuccess.create({
      playerId,
      world: world.uri,
    });

    this.send({
      joinSuccess,
      oneofKind: "joinSuccess",
    });
  }

  leave(uri: string) {
    const world = this.#registry.getWorld(uri);
    if (!world) return;

    this.ws?.unsubscribe(world.topic);

    world.leave(this);
    this.worlds.delete(world);

    const consumers = this.consumers.get(world);
    if (consumers) {
      consumers.forEach((consumer) => consumer.close());
      this.consumers.delete(world);
    }

    const dataConsumers = this.dataConsumers.get(world);
    if (dataConsumers) {
      dataConsumers.forEach((dataConsumer) => dataConsumer.close());
      this.dataConsumers.delete(world);
    }
  }

  sendEvent(data: Uint8Array) {
    this.worlds.forEach((world) => world.sendEvent(this, data));
  }

  chat(message: string) {
    this.worlds.forEach((world) => world.chat(this, message));
  }

  setTransport(type: TransportType, transport: WebRtcTransport) {
    if (type === TransportType.PRODUCER) {
      this.producerTransport = transport;
      return;
    }

    this.consumerTransport = transport;

    if (this.rtpCapabilities) {
      this.#startConsuming();
    }
  }

  async produce(rtpParameters: RtpParameters) {
    if (!this.producerTransport) return;

    try {
      this.producer = await this.producerTransport.produce({
        kind: "audio",
        rtpParameters,
      });

      const producerId = ProducerId.create({
        producerId: this.producer.id,
      });

      this.send({
        oneofKind: "producerId",
        producerId,
      });
    } catch (err) {
      console.warn(err);
    }
  }

  async produceData(sctpStreamParameters: SctpStreamParameters) {
    if (!this.producerTransport) return;

    try {
      this.dataProducer = await this.producerTransport.produceData({
        sctpStreamParameters,
      });

      const dataProducerId = DataProducerId.create({
        dataProducerId: this.dataProducer.id,
      });

      this.send({
        dataProducerId,
        oneofKind: "dataProducerId",
      });
    } catch (err) {
      console.warn(err);
    }
  }

  async consume(producer: Producer, worldUri: string, playerId: number) {
    if (!this.consumerTransport || !this.rtpCapabilities) return;

    try {
      const consumer = await this.consumerTransport.consume({
        paused: true,
        producerId: producer.id,
        rtpCapabilities: this.rtpCapabilities,
      });

      const world = this.#registry.getWorld(worldUri);
      if (!world) return;

      const consumers = this.consumers.get(world) ?? new Map();
      this.consumers.set(world, consumers);

      consumers.set(playerId, consumer);

      const createConsumer = CreateConsumer.create({
        consumerId: consumer.id,
        playerId,
        producerId: producer.id,
        rtpParameters: consumer.rtpParameters,
      });

      this.send({
        createConsumer,
        oneofKind: "createConsumer",
      });
    } catch (err) {
      console.warn(err);
    }
  }

  async consumeData(
    dataProducer: DataProducer,
    worldUri: string,
    playerId: number
  ) {
    if (!this.consumerTransport) return;

    try {
      const dataConsumer = await this.consumerTransport.consumeData({
        dataProducerId: dataProducer.id,
        maxRetransmits: 0,
        ordered: false,
      });
      if (!dataConsumer.sctpStreamParameters) return;

      const world = this.#registry.getWorld(worldUri);
      if (!world) return;

      const dataConsumers = this.dataConsumers.get(world) ?? new Map();
      this.dataConsumers.set(world, dataConsumers);

      dataConsumers.set(playerId, dataConsumer);

      const createDataConsumer = CreateDataConsumer.create({
        dataConsumerId: dataConsumer.id,
        dataProducerId: dataProducer.id,
        playerId,
        sctpStreamParameters: dataConsumer.sctpStreamParameters,
      });

      this.send({
        createDataConsumer,
        oneofKind: "createDataConsumer",
      });
    } catch (err) {
      console.warn(err);
    }
  }

  setPaused(value: boolean) {
    this.consumers.forEach((consumers) => {
      consumers.forEach((consumer) => {
        if (consumer.paused && !consumer.closed) {
          if (value) consumer.pause();
          else consumer.resume();
        }
      });
    });
  }

  #startConsuming() {
    // Consume all other players
    this.worlds.forEach((world) =>
      world.players.forEach((otherPlayer, otherPlayerId) => {
        if (otherPlayer.producer)
          this.consume(otherPlayer.producer, world.uri, otherPlayerId);
        if (otherPlayer.dataProducer)
          this.consumeData(otherPlayer.dataProducer, world.uri, otherPlayerId);
      })
    );
  }

  close() {
    this.ws = null;
    this.worlds.forEach((world) => this.leave(world.uri));
    this.consumerTransport?.close();
    this.producerTransport?.close();
  }
}
