import { MessageSchema } from "@wired-labs/protocol";
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
      const parsed = MessageSchema.fromHost.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { subject, data } = parsed.data;

      switch (subject) {
        case "player_joined": {
          console.info("👋 Player", toHex(data.playerId), "joined");

          const player = new Player(data.playerId);
          localPlayers.push(player);
          setPlayers((players) => [...players, player]);

          if (!data.beforeYou) {
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
          }

          break;
        }

        case "player_left": {
          console.info("👋 Player", toHex(data.playerId), "left");

          const player = localPlayers.find((player) => player.id === data.playerId);

          if (player) {
            player.remove();
            localPlayers.splice(localPlayers.indexOf(player), 1);
            setPlayers((players) => players.filter((player) => player.id !== data.playerId));

            setChatMessages((messages) => [
              ...messages,
              {
                type: "system",
                variant: "player_left",
                playerId: data.playerId,
                id: nanoid(),
                timestamp: Date.now(),
              },
            ]);
          }
          break;
        }

        case "player_address": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) {
            player.address = data.address;
            setPlayers((players) => [...players]);
          }
          break;
        }

        case "player_avatar": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) player.avatar = data.avatar;
          break;
        }

        case "player_grounded": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) player.grounded = data.grounded;
          break;
        }

        case "player_name": {
          const player = localPlayers.find((player) => player.id === data.playerId);
          if (player) {
            player.name = data.name;
            setPlayers((players) => [...players]);
          }
          break;
        }

        case "player_chat": {
          setChatMessages((messages) => [
            ...messages,
            {
              type: "player",
              text: data.text,
              playerId: data.playerId,
              id: nanoid(),
              timestamp: data.timestamp,
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
