import { FromHostMessage } from "protocol";

import { TrpcContext } from "../../client/trpc";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";
import { Player } from "./Player";

export class Players {
  #trpc: TrpcContext;
  #store = new Map<number, Player>();

  constructor(trpc: TrpcContext) {
    this.#trpc = trpc;
  }

  onmessage({ subject, data }: FromHostMessage) {
    switch (subject) {
      case "player_joined": {
        const player = new Player(data.playerId, this.#trpc);
        this.addPlayer(player);
        console.info("👋 Player", player.hexId, "joined");
        break;
      }

      case "player_left": {
        this.removePlayer(data.playerId);
        console.info("👋 Player", numberToHexDisplay(data.playerId), "left");
        break;
      }

      case "player_address": {
        const player = this.#store.get(data.playerId);
        if (player) player.address = data.address;
        break;
      }

      case "player_nickname": {
        const player = this.#store.get(data.playerId);
        if (player) player.nickname = data.name;
        break;
      }

      case "player_chat": {
        const player = this.#store.get(data.playerId);
        if (player) player.sendChatMessage(data.text, data.timestamp);
        break;
      }

      case "player_falling_state": {
        const player = this.#store.get(data.playerId);
        if (player) player.isFalling = data.isFalling;
        break;
      }
    }
  }

  addPlayer(player: Player) {
    // Remove current player if it exists
    this.#store.delete(player.id);
    // Add new player
    this.#store.set(player.id, player);
  }

  removePlayer(playerId: number) {
    this.#store.delete(playerId);
  }

  getPlayer(playerId: number) {
    return this.#store.get(playerId);
  }
}
