import { POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "engine";
import { Device } from "mediasoup-client";
import { Transport } from "mediasoup-client/lib/Transport";
import { fromHostMessageSchema, ToHostMessage } from "protocol";
import { useEffect, useState } from "react";

import { useAppStore } from "../../app/store";
import { trpc } from "../../client/trpc";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { PlayerName } from "../networking/PlayerName";
import { Players } from "../networking/Players";

const PUBLISH_HZ = 15; // X times per second

export function useHost(id: number, host: string) {
  const engine = useAppStore((state) => state.engine);
  const utils = trpc.useContext();

  const [spaceJoined, setSpaceJoined] = useState(false);
  const [reconnectCount, setReconnectCount] = useState(0);

  // Create WebSocket connections
  useEffect(() => {
    if (!engine) return;

    const ws = new WebSocket(host);
    const players = new Players(utils, engine);
    const device = new Device();
    const audioContext = new AudioContext();
    const panners = new Map<number, PannerNode>();

    // Try to play audio
    if (audioContext.state === "suspended") audioContext.resume();

    // Play audio on user interaction
    const play = () => {
      if (audioContext.state === "suspended") audioContext.resume();
      if (audioContext.state === "running") {
        document.removeEventListener("click", play);
        document.removeEventListener("touchstart", play);
      }
    };

    document.addEventListener("click", play);
    document.addEventListener("touchstart", play);

    useAppStore.setState({ ws, players });

    let publishInterval: NodeJS.Timeout | null = null;
    let consumerTransport: Transport | null = null;
    let onProducerId: ((id: string) => void) | null = null;
    let onDataProducerId: ((id: string) => void) | null = null;

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      // Initiate WebRTC connection
      sendToHost({ subject: "get_router_rtp_capabilities", data: null });

      // Join space
      sendToHost({ subject: "join", data: id });

      engine.physics.addEventListener("user_grounded", (event) => {
        sendToHost({ subject: "set_grounded", data: event.data });
      });
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Connection closed");

      // Attempt to reconnect
      setReconnectCount(reconnectCount + 1);
    };

    ws.onmessage = async (event: MessageEvent<string>) => {
      const parsed = fromHostMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      players.onmessage(parsed.data);

      const { subject, data } = parsed.data;

      switch (subject) {
        case "join_success": {
          setSpaceJoined(true);
          console.info(`🌏 Joined space as player ${numberToHexDisplay(data.playerId)}`);
          const name = new PlayerName(data.playerId, utils, engine);
          players.names.set(data.playerId, name);
          useAppStore.setState({ playerId: data.playerId });
          break;
        }

        case "router_rtp_capabilities": {
          if (device.loaded) break;

          // Create device
          await device.load({ routerRtpCapabilities: data });

          // Create transports
          sendToHost({ subject: "create_transport", data: { type: "producer" } });
          sendToHost({ subject: "create_transport", data: { type: "consumer" } });

          // Set rtp capabilities
          sendToHost({ subject: "set_rtp_capabilities", data: device.rtpCapabilities });
          break;
        }

        case "transport_created": {
          // Create local transport
          const transport =
            data.type === "producer"
              ? device.createSendTransport(data.options as any)
              : device.createRecvTransport(data.options as any);

          transport.on("connect", async ({ dtlsParameters }, callback) => {
            sendToHost({ subject: "connect_transport", data: { dtlsParameters, type: data.type } });
            callback();
          });

          transport.on("connectionstatechange", async (state) => {
            console.info(`WebRTC - ${data.type} ${state}`);

            if (state === "connected" && data.type === "producer") {
              const { producerTransport, producedTrack } = useAppStore.getState();

              // Produce audio
              if (producedTrack && producerTransport) {
                const producer = await producerTransport.produce({ track: producedTrack });

                const { micPaused } = useAppStore.getState();

                if (micPaused) producer.pause();
                else producer.resume();

                useAppStore.setState({ producer });
              }
            }
          });

          if (data.type === "producer") {
            useAppStore.setState({ producerTransport: transport });

            transport.on("produce", async ({ kind, rtpParameters }, callback) => {
              if (kind === "video") throw new Error("Video not supported");
              onProducerId = (id) => callback({ id });
              sendToHost({ subject: "produce", data: rtpParameters });
            });

            transport.on("producedata", async ({ sctpStreamParameters }, callback) => {
              onDataProducerId = (id) => callback({ id });
              sendToHost({ subject: "produce_data", data: sctpStreamParameters });
            });

            const dataProducer = await transport.produceData({ ordered: false, maxRetransmits: 0 });

            // Player id = 1 byte (Uint8)
            // Position = 3 * 4 bytes (Int32)
            // Rotation = 4 * 2 bytes (Int16)
            // Total = 21 bytes
            const bytes = 21;
            const buffer = new ArrayBuffer(bytes);
            const view = new DataView(buffer);

            publishInterval = setInterval(() => {
              const { playerId } = useAppStore.getState();
              if (playerId === null || dataProducer.readyState !== "open") return;

              // Set audio listener location
              const camPosX = Atomics.load(engine.cameraPosition, 0) / POSITION_ARRAY_ROUNDING;
              const camPosY = Atomics.load(engine.cameraPosition, 1) / POSITION_ARRAY_ROUNDING;
              const camPosZ = Atomics.load(engine.cameraPosition, 2) / POSITION_ARRAY_ROUNDING;

              const camYaw = Atomics.load(engine.cameraYaw, 0) / ROTATION_ARRAY_ROUNDING;

              const listener = audioContext.listener;

              if (listener.positionX !== undefined) {
                listener.positionX.value = camPosX;
                listener.positionY.value = camPosY;
                listener.positionZ.value = camPosZ;
              } else {
                listener.setPosition(camPosX, camPosY, camPosZ);
              }

              if (listener.forwardX !== undefined) {
                listener.forwardX.value = Math.sin(camYaw);
                listener.forwardZ.value = Math.cos(camYaw);
              } else {
                listener.setOrientation(Math.sin(camYaw), 0, Math.cos(camYaw), 0, 1, 0);
              }

              // Read location
              const posX = Atomics.load(engine.userPosition, 0);
              const posY = Atomics.load(engine.userPosition, 1);
              const posZ = Atomics.load(engine.userPosition, 2);

              const rotX = Atomics.load(engine.userRotation, 0);
              const rotY = Atomics.load(engine.userRotation, 1);
              const rotZ = Atomics.load(engine.userRotation, 2);
              const rotW = Atomics.load(engine.userRotation, 3);

              // Create buffer
              view.setUint8(0, playerId);

              view.setInt32(1, posX, true);
              view.setInt32(5, posY, true);
              view.setInt32(9, posZ, true);

              view.setInt16(13, rotX, true);
              view.setInt16(15, rotY, true);
              view.setInt16(17, rotZ, true);
              view.setInt16(19, rotW, true);

              // Publish buffer
              dataProducer.send(buffer);
            }, 1000 / PUBLISH_HZ);
          }

          if (data.type === "consumer") consumerTransport = transport;
          break;
        }

        case "create_consumer": {
          if (!consumerTransport) throw new Error("Consumer transport not initialized");
          if (consumerTransport.closed) throw new Error("Consumer transport closed");

          const consumer = await consumerTransport.consume({
            id: data.consumerId,
            producerId: data.producerId,
            rtpParameters: data.rtpParameters,
            kind: "audio",
          });

          // Start receiving audio
          sendToHost({ subject: "resume_audio", data: null });
          await consumer.resume();

          // Create audio stream
          const stream = new MediaStream([consumer.track.clone()]);

          // Create audio element
          const audio = new Audio();
          audio.srcObject = stream;
          audio.autoplay = true;

          // Play audio on user interaction
          const play = () => {
            if (audioContext.state === "suspended") audio.play();
            if (audioContext.state === "running") {
              document.removeEventListener("click", play);
              document.removeEventListener("touchstart", play);
            }
          };

          document.addEventListener("click", play);
          document.addEventListener("touchstart", play);

          // Create audio source
          const source = audioContext.createMediaStreamSource(audio.srcObject);

          // Create panner
          const panner = audioContext.createPanner();
          panner.panningModel = "HRTF";
          panner.rolloffFactor = 0.5;

          // Connect nodes
          source.connect(panner);
          panner.connect(audioContext.destination);

          // Store panner
          panners.set(data.playerId, panner);
          break;
        }

        case "create_data_consumer": {
          if (!consumerTransport) throw new Error("Consumer transport not initialized");
          if (consumerTransport.closed) throw new Error("Consumer transport closed");

          const dataConsumer = await consumerTransport.consumeData({
            id: data.consumerId,
            dataProducerId: data.dataProducerId,
            sctpStreamParameters: data.sctpStreamParameters,
          });

          dataConsumer.on("message", async (data: ArrayBuffer | Blob) => {
            const buffer = data instanceof ArrayBuffer ? data : await data.arrayBuffer();

            const view = new DataView(buffer);

            const playerId = view.getUint8(0);

            const posX = view.getInt32(1, true) / POSITION_ARRAY_ROUNDING;
            const posY = view.getInt32(5, true) / POSITION_ARRAY_ROUNDING;
            const posZ = view.getInt32(9, true) / POSITION_ARRAY_ROUNDING;

            const rotX = view.getInt16(13, true) / ROTATION_ARRAY_ROUNDING;
            const rotY = view.getInt16(15, true) / ROTATION_ARRAY_ROUNDING;
            const rotZ = view.getInt16(17, true) / ROTATION_ARRAY_ROUNDING;
            const rotW = view.getInt16(19, true) / ROTATION_ARRAY_ROUNDING;

            // Apply location to audio panner
            const panner = panners.get(playerId);
            if (panner) {
              if (panner.positionX !== undefined) {
                panner.positionX.value = posX;
                panner.positionY.value = posY;
                panner.positionZ.value = posZ;
              } else {
                panner.setPosition(posX, posY, posZ);
              }
            }

            // Send to engine
            const player = engine.player.getPlayer(playerId);
            if (player) {
              player.setPosition(posX, posY, posZ);
              player.setRotation(rotX, rotY, rotZ, rotW);
            }
          });

          break;
        }

        case "producer_id": {
          if (onProducerId) onProducerId(data);
          break;
        }

        case "data_producer_id": {
          if (onDataProducerId) onDataProducerId(data);
          break;
        }
      }
    };

    return () => {
      if (publishInterval) clearInterval(publishInterval);
      ws.close();
      setSpaceJoined(false);
      useAppStore.setState({ ws: null, players: null, playerId: null });
      players.names.forEach((_, id) => engine.player.removePlayer(id));
      document.removeEventListener("click", play);
      document.removeEventListener("touchstart", play);
      audioContext.close();
    };
  }, [engine, utils, id, host, reconnectCount]);

  return { spaceJoined };
}

export function sendToHost(message: ToHostMessage) {
  const { ws } = useAppStore.getState();
  if (!ws || ws.readyState !== ws.OPEN) return;

  ws.send(JSON.stringify(message));
}
