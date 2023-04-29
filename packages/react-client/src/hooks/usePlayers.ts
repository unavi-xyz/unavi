import { ResponseMessageSchema } from "@wired-protocol/types";
import { nanoid } from "nanoid";
import { useEffect } from "react";

import { Player } from "../classes/Player";
import { toHex } from "../utils/toHex";
import { useClient } from "./useClient";

/**
 * Handles the creation and removal of players from the engine.
 *
 * @param ws WebSocket connection
 * @returns List of players and chat messages
 */
export function usePlayers() {
  const { engine, ws, players, setPlayers, setChatMessages } = useClient();

  useEffect(() => {
    if (!ws) return;

    const localPlayers: Player[] = [];

    const onMessage = async (event: MessageEvent) => {
      const parsed = ResponseMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { id, data } = parsed.data;

      switch (id) {
        case "xyz.unavi.world.player.join": {
          console.info("👋 Player", toHex(data.playerId), "joined");

          const player = new Player(data.playerId);
          player.avatar = data.avatar ?? null;
          player.handle = data.handle ?? null;
          player.name = data.name ?? null;
          localPlayers.push(player);
          setPlayers((players) => [...players, player]);

          setChatMessages((messages) => [
            ...messages,
            {
              type: "system",
              variant: "player_joined",
              playerId: data.playerId,
              id: nanoid(),
              timestamp: Date.now(),
            },
          ]);

          break;
        }

        case "xyz.unavi.world.player.leave": {
          console.info("👋 Player", toHex(data), "left");

          const player = localPlayers.find((player) => player.id === data);

          if (player) {
            player.remove();

            // Remove from localPlayers, but keep in state
            // so that the player's name can be displayed in chat
            localPlayers.splice(localPlayers.indexOf(player), 1);

            setChatMessages((messages) => [
              ...messages,
              {
                type: "system",
                variant: "player_left",
                playerId: data,
                id: nanoid(),
                timestamp: Date.now(),
              },
            ]);
          }
          break;
        }
        case "xyz.unavi.world.player.avatar": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) player.avatar = data.avatar;
          break;
        }

        case "xyz.unavi.world.player.handle": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) {
            player.handle = data.handle;
            setPlayers((players) => [...players]);
          }
          break;
        }

        case "xyz.unavi.world.player.name": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) {
            player.name = data.name;
            setPlayers((players) => [...players]);
          }
          break;
        }

        case "xyz.unavi.world.player.grounded": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) player.grounded = data.grounded;
          break;
        }

        case "xyz.unavi.world.chat.message": {
          setChatMessages((messages) => [
            ...messages,
            {
              type: "player",
              text: data.message,
              playerId: data.playerId,
              id: nanoid(),
              timestamp: Date.now(),
            },
          ]);

          break;
        }
      }
    };

    ws.addEventListener("message", onMessage);

    return () => {
      ws.removeEventListener("message", onMessage);

      localPlayers.forEach((player) => player.remove());
      setPlayers([]);
    };
  }, [ws, setPlayers, setChatMessages]);

  useEffect(() => {
    players.forEach((player) => {
      if (player.engine !== engine) player.engine = engine;
    });
  }, [players, engine]);
}
