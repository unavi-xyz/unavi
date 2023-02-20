import { Engine } from "engine";
import { nanoid } from "nanoid";
import { FromHostMessage } from "protocol";

import { TrpcContext } from "../../client/trpc";
import { usePlayStore } from "../store";
import { addChatMessage } from "../utils/addChatMessage";
import { PlayerName } from "./PlayerName";

/**
 * Manages the lifecycle of players.
 * Reads messages from the host and updates the engine.
 */
export class Players {
  #trpc: TrpcContext;
  #engine: Engine;

  names = new Map<number, PlayerName>();

  constructor(trpc: TrpcContext, engine: Engine) {
    this.#trpc = trpc;
    this.#engine = engine;
  }

  onmessage({ subject, data }: FromHostMessage) {
    const { playerId: userId } = usePlayStore.getState();

    switch (subject) {
      case "player_joined": {
        const player = this.#engine.player.addPlayer(data.playerId);
        player.avatar = data.avatar;

        const name = new PlayerName(data.playerId, this.#trpc, this.#engine);
        name.address = data.address;
        name.nickname = data.name;
        this.names.set(data.playerId, name);

        if (!data.beforeYou) {
          addChatMessage({
            type: "system",
            variant: "player_joined",
            id: nanoid(),
            displayName: name.displayName,
            playerId: data.playerId,
            timestamp: Date.now(),
          });
        }

        console.info("👋 Player", name.hexId, "joined");
        break;
      }

      case "player_left": {
        const name = this.names.get(data.playerId);

        if (name) {
          addChatMessage({
            type: "system",
            variant: "player_left",
            id: nanoid(),
            timestamp: Date.now(),
            playerId: data.playerId,
            displayName: name.displayName,
          });

          console.info("👋 Player", name.hexId, "left");

          this.names.delete(data.playerId);
        }

        this.#engine.player.removePlayer(data.playerId);
        break;
      }

      case "player_address": {
        if (data.playerId === userId) break;

        const name = this.names.get(data.playerId);
        if (name) name.address = data.address;
        break;
      }

      case "player_name": {
        if (data.playerId === userId) break;

        const name = this.names.get(data.playerId);
        if (name) name.nickname = data.name;
        break;
      }

      case "player_chat": {
        const name = this.names.get(data.playerId);
        if (!name) break;

        addChatMessage({
          type: "chat",
          id: nanoid(),
          displayName: name.displayName,
          ...data,
        });
        break;
      }

      case "player_grounded": {
        if (data.playerId === userId) break;

        const player = this.#engine.player.getPlayer(data.playerId);
        if (player) player.grounded = data.grounded;
        break;
      }

      case "player_avatar": {
        if (data.playerId === userId) break;

        const player = this.#engine.player.getPlayer(data.playerId);
        if (player) player.avatar = data.avatar;
        break;
      }
    }
  }
}
