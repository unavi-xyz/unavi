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
  const { engine, ws, ethersProvider, players, setPlayers, setChatMessages } = useClient();

  useEffect(() => {
    if (!ws) return;

    const localPlayers: Player[] = [];

    const onMessage = async (event: MessageEvent) => {
      const parsed = ResponseMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { type, data } = parsed.data;

      switch (type) {
        case "player_join": {
          console.info("👋 Player", toHex(data.playerId), "joined");

          const player = new Player(data.playerId);
          player.name = data.name ?? null;
          player.avatar = data.avatar ?? null;
          player.address = data.address ?? null;
          localPlayers.push(player);
          setPlayers((players) => [...players, player]);

          // if (!data.beforeYou) {
          //   setChatMessages((messages) => [
          //     ...messages,
          //     {
          //       type: "system",
          //       variant: "player_joined",
          //       playerId: data.playerId,
          //       id: nanoid(),
          //       timestamp: Date.now(),
          //     },
          //   ]);
          // }

          break;
        }

        case "player_leave": {
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

        case "chat_message": {
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

  useEffect(() => {
    players.forEach((player) => {
      if (player.ethersProvider !== ethersProvider) player.ethersProvider = ethersProvider;
    });
  }, [players, ethersProvider]);
}
