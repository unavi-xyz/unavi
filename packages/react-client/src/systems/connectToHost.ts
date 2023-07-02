import { RequestMessage, ResponseMessageSchema } from "@wired-protocol/types";
import { Device } from "mediasoup-client";
import {
  Consumer,
  DataConsumer,
  DataProducer,
  Producer,
  Transport,
} from "mediasoup-client/lib/types";
import { Query, SystemRes } from "thyseus";

import { WorldJson } from "../components";
import { useClientStore } from "../store";
import { deserializeLocation } from "../utils/deserializeLocation";
import { toHex } from "../utils/toHex";

let chatId = 0;

class LocalRes {
  host = "";
}

export function connectToHost(
  localRes: SystemRes<LocalRes>,
  worlds: Query<WorldJson>
) {
  for (const world of worlds) {
    if (localRes.host === world.host) continue;

    localRes.host = world.host;

    useClientStore.getState().cleanupConnection();

    const prefix = world.host.startsWith("localhost") ? "ws://" : "wss://";
    const ws = new WebSocket(`${prefix}${world.host}`);

    const sendQueue: RequestMessage[] = [];

    const send = (message: RequestMessage) => {
      if (ws.readyState === ws.OPEN) {
        ws.send(JSON.stringify(message));
      } else {
        sendQueue.push(message);
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
        cleanupConnection: () => {},
        playerId: null,
        sendWebRTC: () => {},
        sendWebSockets: () => {},
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
      send({ data: null, id: "xyz.unavi.webrtc.router.rtpCapabilities.get" });

      // Join world
      const uri = useClientStore.getState().worldUri;
      send({ data: uri, id: "xyz.unavi.world.join" });
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Connection closed");
    };

    ws.onerror = (error) => {
      console.error("WebSocket - ⚠️ Connection error", error);
    };

    ws.onmessage = async (event) => {
      const parsed = ResponseMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { data, id } = parsed.data;

      switch (id) {
        case "xyz.unavi.world.joined": {
          console.info(`🌏 Joined world as player ${toHex(data)}`);

          useClientStore.setState({ playerId: data });
          break;
        }

        case "xyz.unavi.world.chat.message": {
          useClientStore.getState().addChatMessage({
            id: chatId++,
            sender: toHex(data.playerId),
            text: data.message,
            timestamp: Date.now(),
            type: "player",
          });
          break;
        }

        case "xyz.unavi.world.player.join": {
          useClientStore.getState().addChatMessage({
            id: chatId++,
            text: `${toHex(data.playerId)} joined the world`,
            timestamp: Date.now(),
            type: "system",
          });
          break;
        }

        case "xyz.unavi.world.player.leave": {
          useClientStore.getState().addChatMessage({
            id: chatId++,
            text: `${toHex(data)} left the world`,
            timestamp: Date.now(),
            type: "system",
          });
          break;
        }

        case "xyz.unavi.webrtc.router.rtpCapabilities": {
          try {
            // Initialize device
            await device.load({ routerRtpCapabilities: data });

            // Create transports
            send({
              data: "producer",
              id: "xyz.unavi.webrtc.transport.create",
            });
            send({
              data: "consumer",
              id: "xyz.unavi.webrtc.transport.create",
            });

            // Set rtp capabilities
            send({
              data: {
                codecs: device.rtpCapabilities.codecs ?? [],
                headerExtensions: device.rtpCapabilities.headerExtensions ?? [],
              },
              id: "xyz.unavi.webrtc.rtpCapabilities.set",
            });
          } catch (error) {
            console.error("Error loading device", error);
          }

          break;
        }

        case "xyz.unavi.webrtc.transport.created": {
          // Create transport
          const transport =
            data.type === "producer"
              ? device.createSendTransport(data.options)
              : device.createRecvTransport(data.options);

          // Connect transport
          transport.on("connect", ({ dtlsParameters }, callback) => {
            send({
              data: { dtlsParameters, type: data.type },
              id: "xyz.unavi.webrtc.transport.connect",
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
              send({ data: rtpParameters, id: "xyz.unavi.webrtc.produce" });
            });

            // producer = await transport.produce({ track });

            transport.on(
              "producedata",
              ({ sctpStreamParameters }, callback) => {
                dataProducerIdCallback = (id: string) => callback({ id });
                send({
                  data: sctpStreamParameters,
                  id: "xyz.unavi.webrtc.produceData",
                });
              }
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

        case "xyz.unavi.webrtc.producer.id": {
          if (producerIdCallback) producerIdCallback(data);
          break;
        }

        case "xyz.unavi.webrtc.dataProducer.id": {
          if (dataProducerIdCallback) dataProducerIdCallback(data);
          break;
        }

        case "xyz.unavi.webrtc.consumer.create": {
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
          send({ data: false, id: "xyz.unavi.webrtc.audio.pause" });
          await consumer.resume();

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

        case "xyz.unavi.webrtc.dataConsumer.create": {
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

            // Listen for data
            dataConsumer.on("message", async (message: ArrayBuffer | Blob) => {
              const buffer =
                message instanceof ArrayBuffer
                  ? message
                  : await message.arrayBuffer();

              const data = deserializeLocation(buffer);
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
