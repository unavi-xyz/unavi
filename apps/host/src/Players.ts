import { FromHostMessage } from "@wired-labs/engine";
import { Consumer } from "mediasoup/node/lib/Consumer";
import { DataConsumer } from "mediasoup/node/lib/DataConsumer";
import { DataProducer } from "mediasoup/node/lib/DataProducer";
import { Producer } from "mediasoup/node/lib/Producer";
import { RtpParameters } from "mediasoup/node/lib/RtpParameters";
import { SctpStreamParameters } from "mediasoup/node/lib/SctpParameters";
import { Transport } from "mediasoup/node/lib/Transport";
import { RtpCapabilities } from "mediasoup-client/lib/RtpParameters";
import { nanoid } from "nanoid";
import uWS from "uWebSockets.js";

import { send } from "./utils/send";

function spaceTopic(spaceId: string) {
  return `space/${spaceId}`;
}

/*
 * Contains logic for managing connected players.
 */
export class Players {
  readonly playerIds = new Map<uWS.WebSocket, string>();
  readonly spaceIds = new Map<uWS.WebSocket, string>();
  readonly names = new Map<uWS.WebSocket, string>();
  readonly avatars = new Map<uWS.WebSocket, string>();
  readonly handles = new Map<uWS.WebSocket, string>();
  readonly rtpCapabilities = new Map<uWS.WebSocket, RtpCapabilities>();
  readonly producerTransports = new Map<uWS.WebSocket, Transport>();
  readonly consumerTransports = new Map<uWS.WebSocket, Transport>();
  readonly producers = new Map<uWS.WebSocket, Producer>();
  readonly consumers = new Map<uWS.WebSocket, Consumer>();
  readonly dataProducers = new Map<uWS.WebSocket, DataProducer>();
  readonly dataConsumers = new Map<uWS.WebSocket, DataConsumer>();

  #server: uWS.TemplatedApp;

  constructor(server: uWS.TemplatedApp) {
    this.#server = server;
  }

  addPlayer(ws: uWS.WebSocket) {
    const playerId = nanoid();

    console.info(`👋 Player ${playerId} connected`);

    this.playerIds.set(ws, playerId);
  }

  removePlayer(ws: uWS.WebSocket) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    console.info(`👋 Player ${playerId} disconnected`);

    this.leaveSpace(ws, false);

    this.playerIds.delete(ws);
    this.spaceIds.delete(ws);
    this.names.delete(ws);
    this.avatars.delete(ws);
    this.handles.delete(ws);
    this.rtpCapabilities.delete(ws);
    this.producerTransports.delete(ws);
    this.consumerTransports.delete(ws);
    this.producers.delete(ws);
    this.consumers.delete(ws);
    this.dataProducers.delete(ws);
    this.dataConsumers.delete(ws);
  }

  joinSpace(ws: uWS.WebSocket, { spaceId }: { spaceId: string }) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    console.info(`🌍 Player ${playerId} joined space ${spaceId}`);

    const name = this.names.get(ws) ?? null;
    const avatar = this.avatars.get(ws) ?? null;
    const handle = this.handles.get(ws) ?? null;

    // Tell everyone that this player joined
    const joinMessage: FromHostMessage = {
      subject: "player_joined",
      data: {
        playerId,
        name,
        avatar,
        handle,
      },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(joinMessage));

    // Subscribe to space topic
    ws.subscribe(spaceTopic(spaceId));

    // Tell this player about everyone else in the space
    this.spaceIds.forEach((otherSpaceId, otherWs) => {
      if (otherSpaceId !== spaceId) return;

      const otherPlayerId = this.playerIds.get(otherWs);
      if (!otherPlayerId) throw new Error("Player not found");

      const otherName = this.names.get(otherWs) ?? null;
      const otherAvatar = this.avatars.get(otherWs) ?? null;
      const otherHandle = this.handles.get(otherWs) ?? null;

      // Send player joined message
      send(ws, {
        subject: "player_joined",
        data: {
          playerId: otherPlayerId,
          name: otherName,
          avatar: otherAvatar,
          handle: otherHandle,
        },
      });

      // Create consumers
      this.createConsumer(ws, otherWs);
      this.createDataConsumer(ws, otherWs);
    });

    // Save space id
    this.spaceIds.set(ws, spaceId);

    // Tell this player their own id
    send(ws, {
      subject: "join_successful",
      data: {
        playerId,
      },
    });

    // Publish WebRTC producers
    this.publishProducer(ws);
    this.publishDataProducer(ws);
  }

  leaveSpace(ws: uWS.WebSocket, isOpen = true) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    console.info(`🌍 Player ${playerId} left space ${spaceId}`);

    // Unsubscribe from space topic if ws connection is still open
    if (isOpen) ws.unsubscribe(spaceTopic(spaceId));
    this.spaceIds.delete(ws);

    // Tell everyone that this player left
    const leaveMessage: FromHostMessage = {
      subject: "player_left",
      data: playerId,
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(leaveMessage));
  }

  publishLocation(
    ws: uWS.WebSocket,
    location: [number, number, number, number, number, number, number]
  ) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's location
    const locationMessage: FromHostMessage = {
      subject: "player_location",
      data: { playerId, location },
    };

    ws.publish(spaceTopic(spaceId), JSON.stringify(locationMessage));
  }

  publishMessage(ws: uWS.WebSocket, message: string) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    const id = nanoid();
    const timestamp = Date.now();

    // Tell everyone in the space about this player's message
    const messageMessage: FromHostMessage = {
      subject: "player_message",
      data: { id, playerId, message, timestamp },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(messageMessage));
  }

  publishFallingState(ws: uWS.WebSocket, isFalling: boolean) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's jump state
    const jumpStateMessage: FromHostMessage = {
      subject: "player_falling_state",
      data: { playerId, isFalling },
    };

    ws.publish(spaceTopic(spaceId), JSON.stringify(jumpStateMessage));
  }

  publishName(ws: uWS.WebSocket, name: string | null) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // Save name
    if (name) this.names.set(ws, name);
    else this.names.delete(ws);

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's name
    const nameMessage: FromHostMessage = {
      subject: "player_name",
      data: { playerId, name },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(nameMessage));
  }

  publishAvatar(ws: uWS.WebSocket, avatar: string | null) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // Save avatar
    if (avatar) this.avatars.set(ws, avatar);
    else this.avatars.delete(ws);

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's avatar
    const avatarMessage: FromHostMessage = {
      subject: "player_avatar",
      data: { playerId, avatar },
    };

    ws.publish(spaceTopic(spaceId), JSON.stringify(avatarMessage));
  }

  publishHandle(ws: uWS.WebSocket, handle: string | null) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // Save handle
    if (handle) this.handles.set(ws, handle);
    else this.handles.delete(ws);

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's handle
    const handleMessage: FromHostMessage = {
      subject: "player_handle",
      data: { playerId, handle },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(handleMessage));
  }

  getPlayerCount(spaceId: string): number {
    let count = 0;

    this.spaceIds.forEach((otherSpaceId) => {
      if (otherSpaceId === spaceId) count++;
    });

    return count;
  }

  setTransport(
    ws: uWS.WebSocket,
    transport: Transport,
    type: "producer" | "consumer"
  ) {
    if (type === "producer") this.producerTransports.set(ws, transport);
    else this.consumerTransports.set(ws, transport);
  }

  async produce(ws: uWS.WebSocket, rtpParameters: RtpParameters) {
    const transport = this.producerTransports.get(ws);
    if (!transport) throw new Error("Producer transport not found");

    const producer = await transport.produce({ kind: "audio", rtpParameters });
    this.producers.set(ws, producer);

    this.publishProducer(ws);

    return producer.id;
  }

  async produceData(
    ws: uWS.WebSocket,
    sctpStreamParameters: SctpStreamParameters
  ) {
    const transport = this.producerTransports.get(ws);
    if (!transport) throw new Error("Producer transport not found");

    const dataProducer = await transport.produceData({ sctpStreamParameters });
    this.dataProducers.set(ws, dataProducer);

    this.publishDataProducer(ws);

    return dataProducer.id;
  }

  async setRtpCapabilities(
    ws: uWS.WebSocket,
    rtpCapabilities: RtpCapabilities
  ) {
    this.rtpCapabilities.set(ws, rtpCapabilities);
  }

  publishProducer(ws: uWS.WebSocket) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Create a consumer for each player in the space
    this.spaceIds.forEach((otherSpaceId, otherWs) => {
      if (otherSpaceId !== spaceId) return;
      this.createConsumer(otherWs, ws);
    });
  }

  publishDataProducer(ws: uWS.WebSocket) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Create a data consumer for each player in the space
    this.spaceIds.forEach((otherSpaceId, otherWs) => {
      if (otherSpaceId !== spaceId) return;
      this.createDataConsumer(otherWs, ws);
    });
  }

  async createConsumer(ws: uWS.WebSocket, otherWs: uWS.WebSocket) {
    const producer = this.producers.get(otherWs);
    if (!producer) return;

    const existingConsumer = this.consumers.get(ws);
    if (existingConsumer) return;

    const otherTransport = this.consumerTransports.get(ws);
    if (!otherTransport) return;

    const rtpCapabilities = this.rtpCapabilities.get(ws);
    if (!rtpCapabilities) return;

    const consumer = await otherTransport.consume({
      producerId: producer.id,
      rtpCapabilities,
    });
    this.consumers.set(ws, consumer);

    send(ws, {
      subject: "create_consumer",
      data: {
        id: consumer.id,
        producerId: producer.id,
        rtpParameters: consumer.rtpParameters,
      },
    });
  }

  async createDataConsumer(ws: uWS.WebSocket, otherWs: uWS.WebSocket) {
    const dataProducer = this.dataProducers.get(otherWs);
    if (!dataProducer) return;

    const existingConsumer = this.dataConsumers.get(ws);
    if (existingConsumer) return;

    const otherTransport = this.consumerTransports.get(ws);
    if (!otherTransport) return;

    const dataConsumer = await otherTransport.consumeData({
      dataProducerId: dataProducer.id,
    });
    if (!dataConsumer.sctpStreamParameters) return;
    this.dataConsumers.set(ws, dataConsumer);

    send(ws, {
      subject: "create_data_consumer",
      data: {
        id: dataConsumer.id,
        dataProducerId: dataProducer.id,
        sctpStreamParameters: dataConsumer.sctpStreamParameters,
      },
    });
  }
}
