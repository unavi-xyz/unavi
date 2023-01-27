import { Engine } from "engine";
import { nanoid } from "nanoid";
import { FromHostMessage } from "protocol";

import { TrpcContext } from "../../client/trpc";
import { addChatMessage } from "../utils/addChatMessage";
import { PlayerName } from "./PlayerName";

export class Players {
  #trpc: TrpcContext;
  #engine: Engine;

  names = new Map<number, PlayerName>();

  constructor(trpc: TrpcContext, engine: Engine) {
    this.#trpc = trpc;
    this.#engine = engine;
  }

  onmessage({ subject, data }: FromHostMessage) {
    switch (subject) {
      case "player_joined": {
        this.#engine.player.addPlayer(data.playerId);

        const name = new PlayerName(data.playerId, this.#trpc);
        this.names.set(data.playerId, name);

        addChatMessage({
          type: "system",
          variant: "player_joined",
          id: nanoid(),
          displayName: name.displayName,
          playerId: data.playerId,
          timestamp: Date.now(),
        });

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
        const name = this.names.get(data.playerId);
        if (name) name.address = data.address;
        break;
      }

      case "player_nickname": {
        const name = this.names.get(data.playerId);
        if (name) name.nickname = data.nickname;
        break;
      }

      case "player_chat": {
        const name = this.names.get(data.playerId);
        if (name) {
          addChatMessage({
            type: "chat",
            id: nanoid(),
            displayName: name.displayName,
            ...data,
          });
        }
        break;
      }

      case "player_grounded": {
        const player = this.#engine.player.getPlayer(data.playerId);
        if (player) player.grounded = data.grounded;
        break;
      }
    }
  }
}
