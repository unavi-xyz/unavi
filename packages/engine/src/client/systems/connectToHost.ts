import {
  CreateTransport,
  GetRouterRtpCapabilities,
  Join,
  Response,
  SendEvent,
  SetRtpCapabilities,
  TransportType,
} from "@wired-protocol/types";
import { Device } from "mediasoup-client";
import {
  Consumer,
  DataConsumer,
  DataProducer,
  Producer,
  Transport,
} from "mediasoup-client/lib/types";
import { Query, SystemRes } from "thyseus";

import { useClientStore } from "../clientStore";
import { WorldJson } from "../components";
import { LOCATION_ROUNDING } from "../constants";
import { toHex } from "../utils/toHex";

let chatId = 0;

class LocalRes {
  host = "";
}

export function connectToHost(
  localRes: SystemRes<LocalRes>,
  worlds: Query<WorldJson>,
) {
  for (const world of worlds) {
    if (localRes.host === world.host) continue;

    localRes.host = world.host;

    useClientStore.getState().cleanupConnection();

    const prefix = world.host.startsWith("localhost") ? "ws://" : "wss://";
    const ws = new WebSocket(`${prefix}${world.host}`);

    const sendQueue: Uint8Array[] = [];

    const send = (data: Uint8Array) => {
      if (ws.readyState === ws.OPEN) {
        const message = SendEvent.create({
          data,
        });
        ws.send(SendEvent.toBinary(message));
      } else {
        sendQueue.push(data);
      }
    };

    useClientStore.setState({ sendWebSockets: send });

    // Create mediasoup device
    const device = new Device();

    let producerIdCallback: ((id: string) => void) | null = null;
    let dataProducerIdCallback: ((id: string) => void) | null = null;

    let consumerTransport: Transport | null = null;
    let producerTransport: Transport | null = null;
    let consumer: Consumer | null = null;
    const producer: Producer | null = null;
    let dataConsumer: DataConsumer | null = null;
    let dataProducer: DataProducer | null = null;

    const cleanupConnection = () => {
      useClientStore.setState({
        cleanupConnection: () => { },
        playerId: null,
        sendWebRTC: () => { },
        sendWebSockets: () => { },
      });

      if (consumerTransport) consumerTransport.close();
      if (producerTransport) producerTransport.close();
      if (consumer) consumer.close();
      // if (producer) producer.close();
      if (dataConsumer) dataConsumer.close();
      if (dataProducer) dataProducer.close();
      if (ws) ws.close();
    };

    useClientStore.setState({ cleanupConnection });

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      // Send all queued messages
      for (const message of sendQueue) {
        send(message);
      }
      sendQueue.length = 0;

      // Initiate WebRTC connection
      send(
        GetRouterRtpCapabilities.toBinary(GetRouterRtpCapabilities.create({})),
      );

      // Join world
      const uri = useClientStore.getState().worldUri;
      const join = Join.create({ world: uri });
      send(Join.toBinary(join));
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Connection closed");
    };

    ws.onerror = (error) => {
      console.error("WebSocket - ⚠️ Connection error", error);
    };

    ws.onmessage = async (e) => {
      const msg = Response.fromBinary(e.data);

      switch (msg.response.oneofKind) {
        case "joinSuccess": {
          const playerId = msg.response.joinSuccess.playerId;
          console.info(`🌏 Joined world as player ${toHex(playerId)}`);
          useClientStore.getState().setPlayerId(playerId);
          break;
        }

        case "chatMessage": {
          useClientStore.getState().addChatMessage({
            id: chatId++,
            playerId: msg.response.chatMessage.playerId,
            text: msg.response.chatMessage.message,
            timestamp: Date.now(),
            type: "player",
          });
          break;
        }

        case "playerJoined": {
          useClientStore.getState().ecsIncoming.push(msg);

          const { setPlayerData } = useClientStore.getState();

          for (const [key, value] of Object.entries(
            msg.response.playerJoined.data,
          )) {
            setPlayerData(msg.response.playerJoined.playerId, key, value);
          }

          const displayName = useClientStore
            .getState()
            .getDisplayName(msg.response.playerJoined.playerId);

          useClientStore.getState().addChatMessage({
            id: chatId++,
            text: `${displayName} joined the world`,
            timestamp: Date.now(),
            type: "system",
          });
          break;
        }

        case "playerLeft": {
          useClientStore.getState().ecsIncoming.push(msg);

          const playerId = msg.response.playerLeft.playerId;

          const displayName = useClientStore
            .getState()
            .getDisplayName(playerId);

          useClientStore.getState().addChatMessage({
            id: chatId++,
            text: `${displayName} left the world`,
            timestamp: Date.now(),
            type: "system",
          });

          useClientStore.getState().playerData.delete(playerId);
          useClientStore.getState().locations.delete(playerId);
          break;
        }

        case "routerRtpCapabilities": {
          if (!msg.response.routerRtpCapabilities.rtpCapabilities) break;

          try {
            // Initialize device
            await device.load({
              routerRtpCapabilities:
                msg.response.routerRtpCapabilities.rtpCapabilities,
            });

            // Create transports
            const createProducerTransport = CreateTransport.create({
              type: TransportType.PRODUCER,
            });
            const createConsumerTransport = CreateTransport.create({
              type: TransportType.CONSUMER,
            });
            send(CreateTransport.toBinary(createProducerTransport));
            send(CreateTransport.toBinary(createConsumerTransport));

            // Set rtp capabilities
            const setRtpCapabilities = SetRtpCapabilities.create({
              rtpCapabilities: device.rtpCapabilities,
            });
            send(SetRtpCapabilities.toBinary(setRtpCapabilities));
          } catch (error) {
            console.error("Error loading device", error);
          }

          break;
        }

        case "com.wired-protocol.webrtc.transport.created": {
          // Create transport
          const transport =
            data.type === "producer"
              ? device.createSendTransport(data.options)
              : device.createRecvTransport(data.options);

          // Connect transport
          transport.on("connect", ({ dtlsParameters }, callback) => {
            send({
              data: { dtlsParameters, type: data.type },
              id: "com.wired-protocol.webrtc.transport.connect",
            });
            callback();
          });

          transport.on("connectionstatechange", (state) => {
            console.info(`WebRTC - ${data.type} ${state}`);
          });

          if (data.type === "consumer") {
            consumerTransport = transport;
          } else {
            producerTransport = transport;

            transport.on("produce", ({ kind, rtpParameters }, callback) => {
              if (kind === "video") {
                console.warn("Video producer not supported");
                return;
              }

              producerIdCallback = (id: string) => callback({ id });
              send({
                data: rtpParameters,
                id: "com.wired-protocol.webrtc.produce",
              });
            });

            // producer = await transport.produce({ track });

            transport.on(
              "producedata",
              ({ sctpStreamParameters }, callback) => {
                dataProducerIdCallback = (id: string) => callback({ id });
                send({
                  data: sctpStreamParameters,
                  id: "com.wired-protocol.webrtc.produceData",
                });
              },
            );

            dataProducer = await transport.produceData({
              maxRetransmits: 0,
              ordered: false,
            });

            useClientStore.setState({
              sendWebRTC: (data) => {
                if (dataProducer?.readyState === "open") {
                  dataProducer.send(data);
                }
              },
            });
          }

          break;
        }

        case "com.wired-protocol.webrtc.producer.id": {
          if (producerIdCallback) producerIdCallback(data);
          break;
        }

        case "com.wired-protocol.webrtc.dataProducer.id": {
          if (dataProducerIdCallback) dataProducerIdCallback(data);
          break;
        }

        case "com.wired-protocol.webrtc.consumer.create": {
          if (!consumerTransport) {
            console.warn("Consumer transport not initialized");
            return;
          }

          consumer = await consumerTransport.consume({
            id: data.consumerId,
            kind: "audio",
            producerId: data.producerId,
            rtpParameters: data.rtpParameters,
          });

          // Start receiving audio
          send({ data: false, id: "com.wired-protocol.webrtc.audio.pause" });
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

        case "com.wired-protocol.webrtc.dataConsumer.create": {
          if (!consumerTransport) {
            console.warn("No consumer transport");
            break;
          }

          try {
            // Create data consumer
            dataConsumer = await consumerTransport.consumeData({
              dataProducerId: data.dataProducerId,
              id: data.dataConsumerId,
              sctpStreamParameters: data.sctpStreamParameters,
            });

            const { locations, lastLocationUpdates } =
              useClientStore.getState();

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

              const location = locations.get(playerId) ?? [];

              location[0] = posX;
              location[1] = posY;
              location[2] = posZ;
              location[3] = rotX;
              location[4] = rotY;
              location[5] = rotZ;
              location[6] = rotW;

              locations.set(playerId, location);
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
