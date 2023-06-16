import { ResponseMessage } from "@wired-protocol/types";
import { Consumer } from "mediasoup/node/lib/Consumer";
import { DataConsumer } from "mediasoup/node/lib/DataConsumer";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";
import {
  RtpCapabilities,
  RtpParameters,
} from "mediasoup/node/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup/node/lib/SctpParameters";
import { Transport } from "mediasoup/node/lib/Transport";

import { Space } from "./Space";
import { SpaceRegistry } from "./SpaceRegistry";
import { uWebSocket } from "./types";

export class Player {
  ws: uWebSocket | null = null;
  #registry: SpaceRegistry;

  spaces = new Set<Space>();
  consumers = new Map<Space, Map<number, Consumer>>();
  dataConsumers = new Map<Space, Map<number, DataConsumer>>();

  #grounded = true;
  #name: string | null = null;
  #handle: string | null = null;
  #avatar: string | null = null;

  #producer: Producer | null = null;
  #dataProducer: DataProducer | null = null;
  #rtpCapabilities: RtpCapabilities | null = null;

  consumerTransport: Transport | null = null;
  producerTransport: Transport | null = null;

  constructor(ws: uWebSocket, registry: SpaceRegistry) {
    this.ws = ws;
    this.#registry = registry;
  }

  send(message: ResponseMessage) {
    this.ws?.send(JSON.stringify(message));
  }

  get name() {
    return this.#name;
  }

  set name(name: string | null) {
    this.#name = name;
    this.spaces.forEach((space) => space.setName(this, name));
  }

  get grounded() {
    return this.#grounded;
  }

  set grounded(grounded: boolean) {
    this.#grounded = grounded;
    this.spaces.forEach((space) => space.setGrounded(this, grounded));
  }

  get handle() {
    return this.#handle;
  }

  set handle(handle: string | null) {
    this.#handle = handle;
    this.spaces.forEach((space) => space.setHandle(this, handle));
  }

  get avatar() {
    return this.#avatar;
  }

  set avatar(avatar: string | null) {
    this.#avatar = avatar;
    this.spaces.forEach((space) => space.setAvatar(this, avatar));
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
      this.spaces.forEach((space) => space.setProducer(this, producer));
  }

  get dataProducer() {
    return this.#dataProducer;
  }

  set dataProducer(dataProducer: DataProducer | null) {
    this.#dataProducer = dataProducer;
    if (dataProducer)
      this.spaces.forEach((space) => space.setDataProducer(this, dataProducer));
  }

  join(uri: string) {
    const space = this.#registry.getOrCreateSpace(uri);
    if (this.spaces.has(space)) return;

    space.join(this);
    this.spaces.add(space);

    const playerId = space.playerId(this);
    if (playerId === undefined) return;

    this.ws?.subscribe(space.topic);

    this.send({ data: playerId, id: "xyz.unavi.world.joined" });
  }

  leave(uri: string) {
    const space = this.#registry.getSpace(uri);
    if (!space) return;

    this.ws?.unsubscribe(space.topic);

    space.leave(this);
    this.spaces.delete(space);

    const consumers = this.consumers.get(space);
    if (consumers) {
      consumers.forEach((consumer) => consumer.close());
      this.consumers.delete(space);
    }

    const dataConsumers = this.dataConsumers.get(space);
    if (dataConsumers) {
      dataConsumers.forEach((dataConsumer) => dataConsumer.close());
      this.dataConsumers.delete(space);
    }
  }

  chat(message: string) {
    this.spaces.forEach((space) => space.chat(this, message));
  }

  setTransport(type: "producer" | "consumer", transport: Transport) {
    if (type === "producer") {
      this.producerTransport = transport;
    } else {
      this.consumerTransport = transport;
      if (this.rtpCapabilities) this.#startConsuming();
    }
  }

  async produce(rtpParameters: RtpParameters) {
    if (!this.producerTransport) return;

    try {
      this.producer = await this.producerTransport.produce({
        kind: "audio",
        rtpParameters,
      });
      this.send({ data: this.producer.id, id: "xyz.unavi.webrtc.producer.id" });
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
      this.send({
        data: this.dataProducer.id,
        id: "xyz.unavi.webrtc.dataProducer.id",
      });
    } catch (err) {
      console.warn(err);
    }
  }

  async consume(producer: Producer, spaceURI: string, playerId: number) {
    if (!this.consumerTransport || !this.rtpCapabilities) return;

    try {
      const consumer = await this.consumerTransport.consume({
        paused: true,
        producerId: producer.id,
        rtpCapabilities: this.rtpCapabilities,
      });

      const space = this.#registry.getSpace(spaceURI);
      if (!space) return;

      const consumers = this.consumers.get(space) ?? new Map();
      this.consumers.set(space, consumers);

      consumers.set(playerId, consumer);

      this.send({
        data: {
          consumerId: consumer.id,
          playerId,
          producerId: producer.id,
          rtpParameters: consumer.rtpParameters,
        },
        id: "xyz.unavi.webrtc.consumer.create",
      });
    } catch (err) {
      console.warn(err);
    }
  }

  async consumeData(
    dataProducer: DataProducer,
    spaceURI: string,
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

      const space = this.#registry.getSpace(spaceURI);
      if (!space) return;

      const dataConsumers = this.dataConsumers.get(space) ?? new Map();
      this.dataConsumers.set(space, dataConsumers);

      dataConsumers.set(playerId, dataConsumer);

      this.send({
        data: {
          dataConsumerId: dataConsumer.id,
          dataProducerId: dataProducer.id,
          playerId,
          sctpStreamParameters: dataConsumer.sctpStreamParameters,
        },
        id: "xyz.unavi.webrtc.dataConsumer.create",
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
    this.spaces.forEach((space) =>
      space.players.forEach((otherPlayer, otherPlayerId) => {
        if (otherPlayer.producer)
          this.consume(otherPlayer.producer, space.uri, otherPlayerId);
        if (otherPlayer.dataProducer)
          this.consumeData(otherPlayer.dataProducer, space.uri, otherPlayerId);
      })
    );
  }

  close() {
    this.ws = null;
    this.spaces.forEach((space) => this.leave(space.uri));
    this.consumerTransport?.close();
    this.producerTransport?.close();
  }
}
