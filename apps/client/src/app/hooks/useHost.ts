import { FromHostMessage, ToHostMessage } from "engine";
import { nanoid } from "nanoid";
import { useEffect, useMemo } from "react";

import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { useAppStore } from "../store";
import { ChatMessage } from "../ui/ChatMessage";

type Player = {
  id: number;
  address: string | null;
  name: string | null;
  avatar: string | null;
};

export function useHost(url: string) {
  const engine = useAppStore((state) => state.engine);

  // Create WebSocket connection
  useEffect(() => {
    if (!engine) return;

    const ws = new WebSocket(url);
    useAppStore.setState({ ws });

    const players = new Map<number, Player>();

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      const { displayName, customAvatar } = useAppStore.getState();

      sendToHost({ subject: "set_name", data: displayName });
      sendToHost({ subject: "set_avatar", data: customAvatar });

      // Start WebRTC connection
      // if (!this.#webRTC) throw new Error("WebRTC not initialized");
      // this.#webRTC.connect();
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Disconnected from host");

      // Remove all players from scene
      engine.renderThread.postMessage({ subject: "clear_players", data: null });

      // if (this.#broadcastInterval) clearInterval(this.#broadcastInterval);

      // if (!this.#hostServer || this.#reconnectCount > 0) return;

      // Try reconnect
      // while (this.#reconnectCount < 10) {
      //   const count = ++this.#reconnectCount;

      //   // Wait a little longer each attempt
      //   const timeoutLength = Math.min(1000 * count);
      //   await new Promise((resolve) => setTimeout(resolve, timeoutLength));

      //   // Test if has been reconnected
      //   if (this.#ws?.readyState === WebSocket.OPEN) return;

      //   // Close preview WebSocket connection
      //   if (this.#ws) this.#ws.close();
      //   this.#ws = null;

      //   // If host server has been removed, give up
      //   if (!this.#hostServer) return;

      //   console.info(`🔄 (${count}) Attempting reconnect to host...`);
      //   this.#connectToHost(spaceId);
      // }

      // console.error("WebSocket - 🪦 Failed to reconnect to host. Giving up.");
      // this.disconnect();
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      const { subject, data }: FromHostMessage = JSON.parse(event.data);

      switch (subject) {
        case "join_successful": {
          console.info(`🌏 Joined space as player ${numberToHexDisplay(data.playerId)}`);

          useAppStore.setState({ playerId: data.playerId });

          const { displayName, customAvatar } = useAppStore.getState();

          const player: Player = {
            id: data.playerId,
            address: null,
            name: displayName,
            avatar: customAvatar,
          };

          // Add to player list
          players.set(data.playerId, player);
          break;
        }

        case "player_joined": {
          console.info(`👋 Player ${numberToHexDisplay(data.playerId)} joined`);

          const player: Player = {
            id: data.playerId,
            address: data.address,
            name: data.name,
            avatar: data.avatar,
          };

          // Add to player list
          players.set(data.playerId, player);

          // Add player to scene
          engine.renderThread.postMessage({ subject: "player_joined", data });

          // Set player name
          engine.renderThread.postMessage({
            subject: "player_name",
            data: {
              playerId: data.playerId,
              name: getUsername(player),
            },
          });

          // Add message to chat if they joined after you
          if (!data.beforeYou)
            addChatMessage({
              type: "system",
              variant: "player_joined",
              id: nanoid(),
              timestamp: Date.now(),
              playerId: data.playerId,
              username: getUsername(player),
            });

          break;
        }

        case "player_left": {
          console.info(`👋 Player ${numberToHexDisplay(data)} left`);

          const player = players.get(data);
          if (!player) throw new Error("Player not found");

          // Remove from player list
          players.delete(data);

          // Remove player from scene
          engine.renderThread.postMessage({ subject: "player_left", data });

          // Add message to chat
          addChatMessage({
            type: "system",
            variant: "player_left",
            id: nanoid(),
            timestamp: Date.now(),
            playerId: data,
            username: getUsername(player),
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
            username: getUsername(player),
          });
          break;
        }

        case "player_falling_state": {
          engine.renderThread.postMessage({
            subject: "set_player_falling_state",
            data,
          });
          break;
        }

        case "player_name": {
          console.info(`📇 Player ${numberToHexDisplay(data.playerId)} is now ${data.name}`);

          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Set player name
          player.name = data.name;

          // Update player name
          engine.renderThread.postMessage({
            subject: "player_name",
            data: {
              playerId: data.playerId,
              name: getUsername(player),
            },
          });
          break;
        }

        case "player_avatar": {
          console.info(`💃 Got custom avatar for ${numberToHexDisplay(data.playerId)}`);

          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Set player avatar
          player.avatar = data.avatar;

          // Load avatar
          engine.renderThread.postMessage({
            subject: "set_player_avatar",
            data: {
              playerId: data.playerId,
              avatar: data.avatar,
            },
          });
          break;
        }

        case "player_address": {
          const { playerId } = useAppStore.getState();
          if (data.playerId !== playerId)
            console.info(`🦩 Player ${numberToHexDisplay(data.playerId)} is now ${data.address}`);

          const player = players.get(data.playerId);
          if (!player) throw new Error("Player not found");

          // Set player address
          player.address = data.address;

          // Update player name
          engine.renderThread.postMessage({
            subject: "player_name",
            data: {
              playerId: data.playerId,
              name: getUsername(player),
            },
          });
          break;
        }
      }
    };

    return () => {
      // Close WebSocket connection
      ws.close();

      // Hack to force close
      ws.onopen = () => {
        ws.close();
      };

      ws.onmessage = () => {
        ws.close();
      };

      ws.onerror = () => {
        ws.close();
      };

      useAppStore.setState({ ws: null });
    };
  }, [engine, url]);

  const connect = useMemo(() => {
    return (id: number) => {
      // Join space
      sendToHost({ subject: "join", data: { id } });
    };
  }, []);

  return {
    connect,
  };
}

export function sendToHost(message: ToHostMessage) {
  const { ws } = useAppStore.getState();
  if (!ws || ws.readyState !== ws.OPEN) throw new Error("WebSocket not initialized");

  ws.send(JSON.stringify(message));
}

function getUsername(player: Player) {
  const username = player.address
    ? player.address.substring(0, 6)
    : player.name
    ? player.name
    : `Guest ${numberToHexDisplay(player.id)}`;

  return username;
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
