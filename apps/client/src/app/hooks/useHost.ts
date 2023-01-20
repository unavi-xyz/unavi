import { Device } from "mediasoup-client";
import { Transport } from "mediasoup-client/lib/Transport";
import { nanoid } from "nanoid";
import { FromHostMessage, ToHostMessage } from "protocol";
import { useEffect, useMemo } from "react";

import { useAppStore } from "../../app/store";
import { trpc } from "../../client/trpc";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { ChatMessage } from "../ui/ChatMessage";

const PUBLISH_HZ = 15; // X times per second

type Player = {
  id: number;
  address: string | null;
  name: string | null;
  username: string;
  avatar: string | null;
};

export function useHost(url: string) {
  const engine = useAppStore((state) => state.engine);

  const utils = trpc.useContext();

  // Create WebSocket connection
  useEffect(() => {
    if (!engine) return;

    const ws = new WebSocket(url);
    useAppStore.setState({ ws });

    const players = new Map<number, Player>();
    const device = new Device();
    const audioContext = new AudioContext();
    const panners = new Map<number, PannerNode>();

    let consumerTransport: Transport | null = null;
    let onProducerId: (({ id }: { id: string }) => void) | null = null;
    let onDataProducerId: (({ id }: { id: string }) => void) | null = null;

    async function updateUsername(player: Player) {
      let oldUsername = player.username;

      if (player.address) {
        // Fetch flamingo profile
        const profile = await utils.social.profile.byAddress.fetch({ address: player.address });

        // Make sure old username is correct, in case it changed while we were fetching
        oldUsername = player.username;

        if (profile?.handle) {
          player.username = profile.handle.string;
        } else {
          player.username = player.address.substring(0, 6);
        }
      } else if (player.name) {
        player.username = player.name;
      } else {
        player.username = `Guest ${numberToHexDisplay(player.id)}`;
      }

      const { playerId } = useAppStore.getState();
      if (player.username !== oldUsername && playerId !== player.id) {
        console.info("📛 Player", numberToHexDisplay(player.id), "is now", player.username);
      }
    }

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      const { displayName, customAvatar } = useAppStore.getState();
      if (displayName) sendToHost({ subject: "set_name", data: displayName });
      if (customAvatar) sendToHost({ subject: "set_avatar", data: customAvatar });

      // Start WebRTC connection
      sendToHost({ subject: "get_router_rtp_capabilities", data: null });

      // Update falling state on change
      // engine.physicsThread.isFalling$.subscribe((isFalling) => {
      //   sendToHost({ subject: "falling_state", data: isFalling });
      // });
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Disconnected from host");

      // Remove all players from scene
      // engine.renderThread.postMessage({ subject: "clear_players", data: null });
    };

    ws.onmessage = async (event: MessageEvent<string>) => {
      const { subject, data }: FromHostMessage = JSON.parse(event.data);

      switch (subject) {
        case "join_successful": {
          console.info(`🌏 Joined space as player ${numberToHexDisplay(data.playerId)}`);

          useAppStore.setState({ playerId: data.playerId });

          const { displayName, customAvatar } = useAppStore.getState();

          const player: Player = {
            id: data.playerId,
            address: null,
            avatar: customAvatar,
            name: displayName,
            username: "",
          };

          await updateUsername(player);

          // Add to player list
          players.set(data.playerId, player);
          break;
        }

        case "player_joined": {
          console.info(`🚪 Player ${numberToHexDisplay(data.playerId)} joined`);

          const player: Player = {
            id: data.playerId,
            address: data.address,
            avatar: data.avatar,
            name: data.name,
            username: "",
          };

          await updateUsername(player);

          // Add to player list
          players.set(data.playerId, player);

          // Add player to scene
          // engine.renderThread.postMessage({ subject: "player_joined", data });

          // Set player name
          // engine.renderThread.postMessage({
          //   subject: "player_name",
          //   data: {
          //     playerId: data.playerId,
          //     name: player.username,
          //   },
          // });

          // Add message to chat if they joined after you
          if (!data.beforeYou)
            addChatMessage({
              type: "system",
              variant: "player_joined",
              id: nanoid(),
              timestamp: Date.now(),
              playerId: data.playerId,
              username: player.username,
            });

          break;
        }

        case "player_left": {
          console.info(`🚪 Player ${numberToHexDisplay(data)} left`);

          const player = players.get(data);
          if (!player) throw new Error("Player not found");

          // Remove from player list
          players.delete(data);

          // Remove player from scene
          // engine.renderThread.postMessage({ subject: "player_left", data });

          // Add message to chat
          addChatMessage({
            type: "system",
            variant: "player_left",
            id: nanoid(),
            timestamp: Date.now(),
            playerId: data,
            username: player.username,
          });
          break;
        }

        case "player_message": {
          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Add message to chat
          addChatMessage({
            type: "chat",
            message: data.message,
            id: data.id,
            timestamp: data.timestamp,
            playerId: data.playerId,
            username: player.username,
          });
          break;
        }

        case "player_falling_state": {
          // engine.renderThread.postMessage({
          //   subject: "set_player_falling_state",
          //   data,
          // });
          break;
        }

        case "player_name": {
          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Set player name
          player.name = data.name;

          // Update player name
          await updateUsername(player);

          // engine.renderThread.postMessage({
          //   subject: "player_name",
          //   data: {
          //     playerId: data.playerId,
          //     name: player.username,
          //   },
          // });
          break;
        }

        case "player_avatar": {
          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Set player avatar
          player.avatar = data.avatar;

          // Load avatar
          // engine.renderThread.postMessage({
          //   subject: "set_player_avatar",
          //   data: {
          //     playerId: data.playerId,
          //     avatar: data.avatar,
          //   },
          // });
          break;
        }

        case "player_address": {
          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Set player address
          player.address = data.address;

          // Update player name
          await updateUsername(player);

          // engine.renderThread.postMessage({
          //   subject: "player_name",
          //   data: {
          //     playerId: data.playerId,
          //     name: player.username,
          //   },
          // });
          break;
        }

        case "router_rtp_capabilities": {
          // Create device
          await device.load({ routerRtpCapabilities: data });

          // Create transports
          sendToHost({
            subject: "create_transport",
            data: { type: "producer" },
          });

          sendToHost({
            subject: "create_transport",
            data: { type: "consumer" },
          });

          // Set rtp capabilities
          sendToHost({
            subject: "set_rtp_capabilities",
            data: {
              rtpCapabilities: device.rtpCapabilities,
            },
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
            sendToHost({
              subject: "connect_transport",
              data: {
                dtlsParameters,
                type: data.type,
              },
            });

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

              sendToHost({
                subject: "produce",
                data: { rtpParameters },
              });
            });

            transport.on("producedata", async ({ sctpStreamParameters }, callback) => {
              onDataProducerId = callback;

              sendToHost({
                subject: "produce_data",
                data: { sctpStreamParameters },
              });
            });

            const dataProducer = await transport.produceData({
              ordered: false,
              maxRetransmits: 0,
            });

            // Player id = 1 byte (Uint8)
            // Position = 3 * 4 bytes (Int32)
            // Rotation = 4 * 2 bytes (Int16)
            // Total = 21 bytes
            const bytes = 21;
            const buffer = new ArrayBuffer(bytes);
            const view = new DataView(buffer);

            // setInterval(() => {
            //   const { playerId } = useAppStore.getState();

            //   const playerPosition = engine.renderThread.playerPosition;
            //   const playerRotation = engine.renderThread.playerRotation;
            //   const cameraPosition = engine.renderThread.cameraPosition;
            //   const cameraRotation = engine.renderThread.cameraRotation;

            //   if (
            //     playerId === null ||
            //     !playerPosition ||
            //     !playerRotation ||
            //     !cameraPosition ||
            //     !cameraRotation ||
            //     dataProducer.readyState !== "open"
            //   )
            //     return;

            //   // Set audio listener location
            //   const camPosX = Atomics.load(cameraPosition, 0);
            //   const camPosY = Atomics.load(cameraPosition, 1);
            //   const camPosZ = Atomics.load(cameraPosition, 2);

            //   const camRotY = Atomics.load(cameraRotation, 1);
            //   const camRotW = Atomics.load(cameraRotation, 3);

            //   const listener = audioContext.listener;

            //   if (listener.positionX !== undefined) {
            //     listener.positionX.value = camPosX / 1000;
            //     listener.positionY.value = camPosY / 1000;
            //     listener.positionZ.value = camPosZ / 1000;
            //   } else {
            //     listener.setPosition(camPosX / 1000, camPosY / 1000, camPosZ / 1000);
            //   }

            //   const yaw = quaternionToYaw(camRotY / 1000, camRotW / 1000);

            //   if (listener.forwardX !== undefined) {
            //     listener.forwardX.value = -Math.sin(yaw);
            //     listener.forwardZ.value = -Math.cos(yaw);
            //   } else {
            //     listener.setOrientation(-Math.sin(yaw), 0, -Math.cos(yaw), 0, 1, 0);
            //   }

            //   // Read location
            //   const posX = Atomics.load(playerPosition, 0);
            //   const posY = Atomics.load(playerPosition, 1);
            //   const posZ = Atomics.load(playerPosition, 2);

            //   const rotX = Atomics.load(playerRotation, 0);
            //   const rotY = Atomics.load(playerRotation, 1);
            //   const rotZ = Atomics.load(playerRotation, 2);
            //   const rotW = Atomics.load(playerRotation, 3);

            //   // Create buffer
            //   view.setUint8(0, playerId);

            //   view.setInt32(1, posX, true);
            //   view.setInt32(5, posY, true);
            //   view.setInt32(9, posZ, true);

            //   view.setInt16(13, rotX, true);
            //   view.setInt16(15, rotY, true);
            //   view.setInt16(17, rotZ, true);
            //   view.setInt16(19, rotW, true);

            //   // Publish buffer
            //   dataProducer.send(buffer);
            // }, 1000 / PUBLISH_HZ);
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
          sendToHost({
            subject: "resume_audio",
            data: null,
          });

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

            // Apply location to audio panner
            const view = new DataView(buffer);
            const playerId = view.getUint8(0);

            const panner = panners.get(playerId);
            if (panner) {
              if (panner.positionX !== undefined) {
                panner.positionX.value = view.getInt32(1, true) / 1000;
                panner.positionY.value = view.getInt32(5, true) / 1000;
                panner.positionZ.value = view.getInt32(9, true) / 1000;
              } else {
                panner.setPosition(
                  view.getInt32(1, true) / 1000,
                  view.getInt32(5, true) / 1000,
                  view.getInt32(9, true) / 1000
                );
              }
            }

            // Send to renderThread
            // engine.renderThread.postMessage({
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
      // Close WebSocket connection
      ws.close();

      // Close WebRTC transports
      const { producerTransport } = useAppStore.getState();
      if (producerTransport) producerTransport.close();
      if (consumerTransport) consumerTransport.close();

      // Reset state
      useAppStore.setState({ ws: null });
    };
  }, [engine, utils, url]);

  const connect = useMemo(() => {
    return (id: number) => {
      // Join space
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

function addChatMessage(message: ChatMessage) {
  const { chatMessages } = useAppStore.getState();

  const newChatMessages = [...chatMessages, message];

  // Sort by timestamp
  newChatMessages.sort((a, b) => b.timestamp - a.timestamp);

  // Limit to 50 messages
  newChatMessages.splice(50, newChatMessages.length - 50);

  useAppStore.setState({ chatMessages: newChatMessages });
}
