import { POSITION_ARRAY_ROUNDING } from "engine/src/constants";
import { Device } from "mediasoup-client";
import { Transport } from "mediasoup-client/lib/Transport";
import { fromHostMessageSchema, ToHostMessage } from "protocol";
import { useEffect, useMemo } from "react";

import { useAppStore } from "../../app/store";
import { trpc } from "../../client/trpc";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { Player } from "../networking/Player";
import { Players } from "../networking/Players";

const PUBLISH_HZ = 15; // X times per second

export function useHost(url: string) {
  const engine = useAppStore((state) => state.engine);
  const utils = trpc.useContext();

  // Create WebSocket connection
  useEffect(() => {
    if (!engine) return;

    const ws = new WebSocket(url);
    const players = new Players(utils);
    const device = new Device();
    const audioContext = new AudioContext();
    const panners = new Map<number, PannerNode>();

    useAppStore.setState({ ws, players });

    let publishInterval: NodeJS.Timeout | null = null;
    let consumerTransport: Transport | null = null;
    let onProducerId: (({ id }: { id: string }) => void) | null = null;
    let onDataProducerId: (({ id }: { id: string }) => void) | null = null;

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      // Initiate WebRTC connection
      sendToHost({ subject: "get_router_rtp_capabilities", data: null });
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Disconnected from host");
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
          console.info(`🌏 Joined space as player ${numberToHexDisplay(data.playerId)}`);
          const player = new Player(data.playerId, utils);
          players.addPlayer(player);
          useAppStore.setState({ playerId: data.playerId });
          break;
        }

        case "router_rtp_capabilities": {
          // Create device
          await device.load({ routerRtpCapabilities: data.rtpCapabilities });

          // Create transports
          sendToHost({ subject: "create_transport", data: { type: "producer" } });
          sendToHost({ subject: "create_transport", data: { type: "consumer" } });

          // Set rtp capabilities
          sendToHost({
            subject: "set_rtp_capabilities",
            data: { rtpCapabilities: device.rtpCapabilities },
          });
          break;
        }

        case "transport_created": {
          // Create local transport
          const transport =
            data.type === "producer"
              ? device.createSendTransport(data.options)
              : device.createRecvTransport(data.options);

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
              onProducerId = callback;
              sendToHost({ subject: "produce", data: { rtpParameters } });
            });

            transport.on("producedata", async ({ sctpStreamParameters }, callback) => {
              onDataProducerId = callback;
              sendToHost({ subject: "produce_data", data: { sctpStreamParameters } });
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

              const positionArray = engine.physics.positionArray;
              const rotationArray = engine.physics.rotationArray;

              if (
                playerId === null ||
                dataProducer.readyState !== "open" ||
                !positionArray ||
                !rotationArray
              )
                return;

              // // Set audio listener location
              // const camPosX = Atomics.load(cameraPosition, 0);
              // const camPosY = Atomics.load(cameraPosition, 1);
              // const camPosZ = Atomics.load(cameraPosition, 2);

              // const camRotY = Atomics.load(cameraRotation, 1);
              // const camRotW = Atomics.load(cameraRotation, 3);

              // const listener = audioContext.listener;

              // if (listener.positionX !== undefined) {
              //   listener.positionX.value = camPosX / 1000;
              //   listener.positionY.value = camPosY / 1000;
              //   listener.positionZ.value = camPosZ / 1000;
              // } else {
              //   listener.setPosition(camPosX / 1000, camPosY / 1000, camPosZ / 1000);
              // }

              // const yaw = quaternionToYaw(camRotY / 1000, camRotW / 1000);

              // if (listener.forwardX !== undefined) {
              //   listener.forwardX.value = -Math.sin(yaw);
              //   listener.forwardZ.value = -Math.cos(yaw);
              // } else {
              //   listener.setOrientation(-Math.sin(yaw), 0, -Math.cos(yaw), 0, 1, 0);
              // }

              // Read location
              const posX = Atomics.load(positionArray, 0);
              const posY = Atomics.load(positionArray, 1);
              const posZ = Atomics.load(positionArray, 2);

              // const rotX = Atomics.load(rotationArray, 0);
              // const rotY = Atomics.load(rotationArray, 1);
              // const rotZ = Atomics.load(rotationArray, 2);
              // const rotW = Atomics.load(rotationArray, 3);

              // Create buffer
              view.setUint8(0, playerId);

              view.setInt32(1, posX, true);
              view.setInt32(5, posY, true);
              view.setInt32(9, posZ, true);

              // view.setInt16(13, rotX, true);
              // view.setInt16(15, rotY, true);
              // view.setInt16(17, rotZ, true);
              // view.setInt16(19, rotW, true);

              // Publish buffer
              dataProducer.send(buffer);
            }, 1000 / PUBLISH_HZ);
          }

          if (data.type === "consumer") {
            consumerTransport = transport;

            sendToHost({
              subject: "ready_to_consume",
              data: true,
            });
          }

          break;
        }

        case "create_consumer": {
          if (!consumerTransport) throw new Error("Consumer transport not initialized");
          if (consumerTransport.closed) throw new Error("Consumer transport closed");

          const consumer = await consumerTransport.consume({
            id: data.id,
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

          // Create audio source
          const source = audioContext.createMediaStreamSource(audio.srcObject);

          // Create panner
          const panner = audioContext.createPanner();
          panner.panningModel = "HRTF";
          panner.rolloffFactor = 0.5;

          // Connect nodes
          source.connect(panner);
          panner.connect(audioContext.destination);

          if (audioContext.state === "suspended") audioContext.resume();

          // Play audio on user interaction
          const play = async () => {
            if (audioContext.state === "suspended") await audioContext.resume();
            document.removeEventListener("click", play);
          };
          document.addEventListener("click", play);

          // Store panner
          panners.set(data.playerId, panner);
          break;
        }

        case "create_data_consumer": {
          if (!consumerTransport) throw new Error("Consumer transport not initialized");
          if (consumerTransport.closed) throw new Error("Consumer transport closed");

          const dataConsumer = await consumerTransport.consumeData({
            id: data.id,
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
            // engine.render.send({
            //   subject: "player_location",
            //   data: buffer,
            // });
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
      useAppStore.setState({ ws: null, players: null, playerId: null });
    };
  }, [engine, utils, url]);

  const connect = useMemo(() => {
    return (id: number) => {
      sendToHost({ subject: "join", data: { id } });
    };
  }, []);

  return { connect };
}

export function sendToHost(message: ToHostMessage) {
  const { ws } = useAppStore.getState();
  if (!ws || ws.readyState !== ws.OPEN) return;

  ws.send(JSON.stringify(message));
}
