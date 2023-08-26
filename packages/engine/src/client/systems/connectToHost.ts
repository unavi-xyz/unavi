import { EditorEvent } from "@unavi/protocol";
import {
  fromMediasoupDtlsParameters,
  fromMediasoupRtpCapabilities,
  fromMediasoupRtpParameters,
  toMediasoupRtpCapabilities,
  toMediasoupRtpParameters,
  toMediasoupTransportOptions,
} from "@unavi/utils";
import {
  ConnectTransport,
  CreateTransport,
  Event,
  GetRouterRtpCapabilities,
  Join,
  PauseAudio,
  Produce,
  ProduceData,
  Request,
  Response,
  SendEvent,
  SetRtpCapabilities,
  TransportType,
} from "@wired-protocol/types";
import { atom } from "jotai";
import { Warehouse } from "lattice-engine/core";
import { Device } from "mediasoup-client";
import {
  Consumer,
  DataConsumer,
  DataProducer,
  Producer,
  Transport,
} from "mediasoup-client/lib/types";
import { Query, Res, SystemRes } from "thyseus";

import { AtomStore } from "../../AtomStore";
import { WorldJson } from "../components";
import { LOCATION_ROUNDING } from "../constants";
import { ChatMessage } from "../types";
import { toHex } from "../utils/toHex";
import { ecsEventStore } from "./sendEvents";

const consumerTransportAtom = atom<Transport | null>(null);
const producerTransportAtom = atom<Transport | null>(null);
const consumerAtom = atom<Consumer | null>(null);
const producerAtom = atom<Producer | null>(null);
const dataConsumerAtom = atom<DataConsumer | null>(null);
const dataProducerAtom = atom<DataProducer | null>(null);

const connectSendCallbackAtom = atom<() => void>(() => {});
const connectRecvCallbackAtom = atom<() => void>(() => {});
const connectSendErrbackAtom = atom<(e: Error) => void>(() => {});
const connectRecvErrbackAtom = atom<(e: Error) => void>(() => {});
const producerIdCallbackAtom = atom<(producerId: string) => void>(() => {});
const dataProducerIdCallbackAtom = atom<(dataProducerId: string) => void>(
  () => {}
);

export class ConnectionStore extends AtomStore {
  worldUri = atom("");
  playerId = atom<number | null>(null);

  nickname = atom("");
  did = atom("");
  avatar = atom("");
  playerData = atom(new Map<number, Record<string, string>>());

  locations = atom(new Map<number, number[]>());
  lastLocationUpdates = atom(new Map<number, number>());

  constructor() {
    super();

    let prevPlayerId: number | null = null;

    this.store.sub(this.playerId, () => {
      const playerId = this.get(this.playerId);
      const playerData = this.get(this.playerData);

      if (prevPlayerId !== null) {
        playerData.delete(prevPlayerId);
      }

      if (playerId !== null) {
        this.setPlayerData(playerId, "avatar", this.get(this.avatar));
        this.setPlayerData(playerId, "did", this.get(this.did));
        this.setPlayerData(playerId, "nickname", this.get(this.nickname));
      }

      prevPlayerId = playerId;
    });
  }

  setPlayerData(playerId: number, key: string, value: string) {
    const playerData = this.get(this.playerData);
    const player = playerData.get(playerId);

    if (!player) {
      playerData.set(playerId, { [key]: value });
    } else {
      player[key] = value;
    }
  }

  ws = atom<WebSocket | null>(null);
  wsMessageQueue: Request["message"][] = [];
  sendWs(message: Request["message"]) {
    const msg = Request.create({
      message,
    });

    const ws = this.get(this.ws);

    if (!ws || ws.readyState !== ws.OPEN) {
      this.wsMessageQueue.push(message);
      return;
    }

    const bytes = Request.toBinary(msg);
    ws.send(bytes);
  }

  sendWebRTC(message: ArrayBuffer) {
    const dataProducer = this.get(dataProducerAtom);
    if (!dataProducer) return;

    dataProducer.send(message);
  }

  mirrorEvent(editorEvent: EditorEvent) {
    const data = EditorEvent.toBinary(editorEvent);

    // Send to self
    const playerId = this.get(this.playerId);
    if (playerId === null) return;

    const event = Event.create({ data, playerId });
    const response = Response.create({
      response: { event, oneofKind: "event" },
    });

    const ecsIncoming = ecsEventStore.get(ecsEventStore.ecsIncoming);
    ecsIncoming.push(response);

    // Send to others
    const sendEvent = SendEvent.create({ data });
    this.sendWs({ oneofKind: "sendEvent", sendEvent });
  }

  chatMessages = atom<ChatMessage[]>([]);
  chatMessageId = 0;
  addChatMessage(message: ChatMessage) {
    const chatMessages = this.get(this.chatMessages);
    chatMessages.push(message);

    if (chatMessages.length > 100) {
      chatMessages.shift();
    }

    this.set(this.chatMessages, chatMessages);
  }

  getDisplayName(playerId: number) {
    const playerData = this.get(this.playerData).get(playerId);

    const did = playerData?.did;
    const nickname = playerData?.nickname;

    if (did) {
      // TODO: Resolve profile
    }

    if (nickname) {
      return nickname;
    }

    return `Guest ${toHex(playerId)}`;
  }

  closeConnection() {
    const ws = this.get(this.ws);
    if (ws) {
      ws.close();
    }

    const consumerTransport = this.get(consumerTransportAtom);
    if (consumerTransport) {
      consumerTransport.close();
    }

    const producerTransport = this.get(producerTransportAtom);
    if (producerTransport) {
      producerTransport.close();
    }

    const consumer = this.get(consumerAtom);
    if (consumer) {
      consumer.close();
    }

    const producer = this.get(producerAtom);
    if (producer) {
      producer.close();
    }

    const dataConsumer = this.get(dataConsumerAtom);
    if (dataConsumer) {
      dataConsumer.close();
    }

    const dataProducer = this.get(dataProducerAtom);
    if (dataProducer) {
      dataProducer.close();
    }

    this.set(this.playerId, null);
    this.set(consumerTransportAtom, null);
    this.set(producerTransportAtom, null);
    this.set(consumerAtom, null);
    this.set(producerAtom, null);
    this.set(dataConsumerAtom, null);
    this.set(dataProducerAtom, null);
    this.set(this.ws, null);
  }
}

export const connectionStore = new ConnectionStore();

class LocalRes {
  host = "";
}

export function connectToHost(
  warehouse: Res<Warehouse>,
  localRes: SystemRes<LocalRes>,
  worlds: Query<WorldJson>
) {
  for (const world of worlds) {
    const host = world.host.read(warehouse) ?? "";
    if (localRes.host === host) continue;

    localRes.host = host;

    connectionStore.closeConnection();

    const prefix = host.startsWith("localhost") ? "ws://" : "wss://";
    const ws = new WebSocket(`${prefix}${world.host}`);

    // Create mediasoup device
    const device = new Device();

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      // Send all queued messages
      for (const message of connectionStore.wsMessageQueue) {
        connectionStore.sendWs(message);
      }
      connectionStore.wsMessageQueue = [];

      // Initiate WebRTC connection
      const getRouterRtpCapabilities = GetRouterRtpCapabilities.create();
      connectionStore.sendWs({
        getRouterRtpCapabilities,
        oneofKind: "getRouterRtpCapabilities",
      });

      // Join world
      const uri = connectionStore.get(connectionStore.worldUri);
      const join = Join.create({ world: uri });
      connectionStore.sendWs({ join, oneofKind: "join" });
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Connection closed");
    };

    ws.onerror = (error) => {
      console.error("WebSocket - ⚠️ Connection error", error);
    };

    ws.onmessage = async (e) => {
      if (!(e.data instanceof Blob)) {
        console.warn("Unexpected message type", e.data);
        return;
      }

      const buffer = await e.data.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      const msg = Response.fromBinary(bytes);

      switch (msg.response.oneofKind) {
        case "joinSuccess": {
          const playerId = msg.response.joinSuccess.playerId;
          console.info(`🌏 Joined world as player ${toHex(playerId)}`);
          connectionStore.set(connectionStore.playerId, playerId);
          break;
        }

        case "chatMessage": {
          connectionStore.addChatMessage({
            id: connectionStore.chatMessageId++,
            playerId: msg.response.chatMessage.playerId,
            text: msg.response.chatMessage.message,
            timestamp: Date.now(),
            type: "player",
          });
          break;
        }

        case "playerJoined": {
          const ecsIncoming = ecsEventStore.get(ecsEventStore.ecsIncoming);
          ecsIncoming.push(msg);

          const playerId = msg.response.playerJoined.playerId;

          for (const [key, value] of Object.entries(
            msg.response.playerJoined.data
          )) {
            connectionStore.setPlayerData(playerId, key, value);
          }

          const displayName = connectionStore.getDisplayName(playerId);

          connectionStore.addChatMessage({
            id: connectionStore.chatMessageId++,
            text: `${displayName} joined the world`,
            timestamp: Date.now(),
            type: "system",
          });
          break;
        }

        case "playerLeft": {
          const ecsIncoming = ecsEventStore.get(ecsEventStore.ecsIncoming);
          ecsIncoming.push(msg);

          const playerId = msg.response.playerLeft.playerId;
          const displayName = connectionStore.getDisplayName(playerId);

          connectionStore.addChatMessage({
            id: connectionStore.chatMessageId++,
            text: `${displayName} left the world`,
            timestamp: Date.now(),
            type: "system",
          });

          connectionStore.get(connectionStore.playerData).delete(playerId);
          connectionStore.get(connectionStore.locations).delete(playerId);
          break;
        }

        case "playerData": {
          connectionStore.setPlayerData(
            msg.response.playerData.playerId,
            msg.response.playerData.key,
            msg.response.playerData.value
          );
          break;
        }

        case "routerRtpCapabilities": {
          if (!msg.response.routerRtpCapabilities.rtpCapabilities) {
            console.warn("Router RTP capabilities are undefined");
            break;
          }

          try {
            // Initialize device
            await device.load({
              routerRtpCapabilities: toMediasoupRtpCapabilities(
                msg.response.routerRtpCapabilities.rtpCapabilities
              ),
            });

            // Create transports
            const createProducerTransport = CreateTransport.create({
              type: TransportType.PRODUCER,
            });
            const createConsumerTransport = CreateTransport.create({
              type: TransportType.CONSUMER,
            });

            connectionStore.sendWs({
              createTransport: createProducerTransport,
              oneofKind: "createTransport",
            });
            connectionStore.sendWs({
              createTransport: createConsumerTransport,
              oneofKind: "createTransport",
            });

            // Set rtp capabilities
            const setRtpCapabilities = SetRtpCapabilities.create({
              rtpCapabilities: fromMediasoupRtpCapabilities(
                device.rtpCapabilities
              ),
            });
            connectionStore.sendWs({
              oneofKind: "setRtpCapabilities",
              setRtpCapabilities,
            });
          } catch (error) {
            console.error("Error loading device", error);
          }

          break;
        }

        case "transportCreated": {
          if (!msg.response.transportCreated.options) {
            console.warn("Transport options are undefined");
            break;
          }

          // Create transport
          const options = toMediasoupTransportOptions(
            msg.response.transportCreated.options
          );

          const transportType = msg.response.transportCreated.type;
          const isProducer = transportType === TransportType.PRODUCER;

          const transport = isProducer
            ? device.createSendTransport(options)
            : device.createRecvTransport(options);

          // Connect transport
          transport.on("connect", ({ dtlsParameters }, callback, errback) => {
            if (isProducer) {
              connectionStore.set(connectSendCallbackAtom, callback);
              connectionStore.set(connectSendErrbackAtom, errback);
            } else {
              connectionStore.set(connectRecvCallbackAtom, callback);
              connectionStore.set(connectRecvErrbackAtom, errback);
            }

            const connect = ConnectTransport.create({
              dtlsParameters: fromMediasoupDtlsParameters(dtlsParameters),
              type: isProducer
                ? TransportType.PRODUCER
                : TransportType.CONSUMER,
            });

            connectionStore.sendWs({
              connectTransport: connect,
              oneofKind: "connectTransport",
            });
          });

          transport.on("connectionstatechange", (state) => {
            const strType = isProducer ? "Producer" : "Consumer";
            console.info(`WebRTC - ${strType} ${state}`);
          });

          if (!isProducer) {
            connectionStore.set(consumerTransportAtom, transport);
          } else {
            connectionStore.set(producerTransportAtom, transport);

            transport.on("produce", ({ kind, rtpParameters }, callback) => {
              if (kind === "video") {
                console.warn("Video producer not supported");
                return;
              }

              connectionStore.set(producerIdCallbackAtom, (id: string) =>
                callback({ id })
              );

              const produce = Produce.create({
                rtpParameters: fromMediasoupRtpParameters(rtpParameters),
              });

              connectionStore.sendWs({ oneofKind: "produce", produce });
            });

            // producer = await transport.produce({ track });

            transport.on(
              "producedata",
              ({ sctpStreamParameters }, callback) => {
                connectionStore.set(dataProducerIdCallbackAtom, (id: string) =>
                  callback({ id })
                );

                const produceData = ProduceData.create({
                  sctpStreamParameters,
                });

                connectionStore.sendWs({
                  oneofKind: "produceData",
                  produceData,
                });
              }
            );

            const dataProducer = await transport.produceData({
              maxRetransmits: 0,
              ordered: false,
            });
            connectionStore.set(dataProducerAtom, dataProducer);
          }

          break;
        }

        case "transportConnected": {
          const transportType = msg.response.transportConnected.type;
          const success = msg.response.transportConnected.success;

          if (success) {
            if (transportType === TransportType.PRODUCER) {
              const connectSendCallback = connectionStore.get(
                connectSendCallbackAtom
              );
              connectSendCallback();
            } else {
              const connectRecvCallback = connectionStore.get(
                connectRecvCallbackAtom
              );
              connectRecvCallback();
            }
          } else {
            const err = new Error("Transport connection failed");
            if (transportType === TransportType.PRODUCER) {
              const connectSendErrback = connectionStore.get(
                connectSendErrbackAtom
              );
              connectSendErrback(err);
            } else {
              const connectRecvErrback = connectionStore.get(
                connectRecvErrbackAtom
              );
              connectRecvErrback(err);
            }
          }

          break;
        }

        case "producerId": {
          const producerIdCallback = connectionStore.get(
            producerIdCallbackAtom
          );
          producerIdCallback(msg.response.producerId.producerId);
          break;
        }

        case "dataProducerId": {
          const dataProducerIdCallback = connectionStore.get(
            dataProducerIdCallbackAtom
          );
          dataProducerIdCallback(msg.response.dataProducerId.dataProducerId);
          break;
        }

        case "createConsumer": {
          const consumerTransport = connectionStore.get(consumerTransportAtom);

          if (!consumerTransport) {
            console.warn("Consumer transport not initialized");
            return;
          }

          if (!msg.response.createConsumer.rtpParameters) {
            console.warn("Did not receive consumer rtp parameters");
            return;
          }

          const consumer = await consumerTransport.consume({
            id: msg.response.createConsumer.consumerId,
            kind: "audio",
            producerId: msg.response.createConsumer.producerId,
            rtpParameters: toMediasoupRtpParameters(
              msg.response.createConsumer.rtpParameters
            ),
          });
          connectionStore.set(consumerAtom, consumer);

          // Start receiving audio
          const pauseAudio = PauseAudio.create({
            paused: false,
          });
          connectionStore.sendWs({ oneofKind: "pauseAudio", pauseAudio });

          consumer.resume();

          // Create audio stream
          const stream = new MediaStream([consumer.track.clone()]);

          // Create audio element
          const audio = new Audio();
          audio.srcObject = stream;
          audio.autoplay = true;

          // // Play audio on user interaction
          // const play = () => {
          //   if (engine.audio.context.state === "suspended") audio.play();
          //   if (engine.audio.context.state === "running") {
          //     document.removeEventListener("click", play);
          //     document.removeEventListener("touchstart", play);
          //   }
          // };

          // document.addEventListener("click", play);
          // document.addEventListener("touchstart", play);

          // // Create panner
          // const panner = engine.audio.createAudioPanner();
          // panner.rolloffFactor = 0.5;

          // // Create audio source
          // const source = engine.audio.context.createMediaStreamSource(
          //   audio.srcObject
          // );
          // source.connect(panner);

          // // Store panner
          // panners.set(data.playerId, panner);
          break;
        }

        case "createDataConsumer": {
          const consumerTransport = connectionStore.get(consumerTransportAtom);

          if (!consumerTransport) {
            console.warn("No consumer transport");
            break;
          }

          if (!msg.response.createDataConsumer.sctpStreamParameters) {
            console.warn(
              "Did not receive data consumer sctp stream parameters"
            );
            break;
          }

          try {
            // Create data consumer
            const dataConsumer = await consumerTransport.consumeData({
              dataProducerId: msg.response.createDataConsumer.dataProducerId,
              id: msg.response.createDataConsumer.dataConsumerId,
              sctpStreamParameters:
                msg.response.createDataConsumer.sctpStreamParameters,
            });
            connectionStore.set(dataConsumerAtom, dataConsumer);

            // Listen for data
            dataConsumer.on("message", async (message: ArrayBuffer | Blob) => {
              const buffer =
                message instanceof ArrayBuffer
                  ? message
                  : await message.arrayBuffer();

              const view = new DataView(buffer);

              const playerId = view.getUint8(0);

              const posX = view.getInt32(1) / LOCATION_ROUNDING.POSITION;
              const posY = view.getInt32(5) / LOCATION_ROUNDING.POSITION;
              const posZ = view.getInt32(9) / LOCATION_ROUNDING.POSITION;

              const rotX = view.getInt16(13) / LOCATION_ROUNDING.ROTATION;
              const rotY = view.getInt16(15) / LOCATION_ROUNDING.ROTATION;
              const rotZ = view.getInt16(17) / LOCATION_ROUNDING.ROTATION;
              const rotW = view.getInt16(19) / LOCATION_ROUNDING.ROTATION;

              const locations = connectionStore.get(connectionStore.locations);
              const location = locations.get(playerId) ?? [];

              location[0] = posX;
              location[1] = posY;
              location[2] = posZ;
              location[3] = rotX;
              location[4] = rotY;
              location[5] = rotZ;
              location[6] = rotW;

              locations.set(playerId, location);

              const lastLocationUpdates = connectionStore.get(
                connectionStore.lastLocationUpdates
              );
              lastLocationUpdates.set(playerId, performance.now());
            });
          } catch (error) {
            console.error("Error consuming data", error);
          }

          break;
        }
      }
    };
  }
}
