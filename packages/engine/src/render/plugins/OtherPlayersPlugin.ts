import { Group, Scene } from "three";

import { OtherPlayer } from "../classes/OtherPlayer";
import { ToRenderMessage } from "../types";

export class OtherPlayersPlugin {
  #scene: Scene;
  #players = new Map<string, OtherPlayer>();

  #playerGroup = new Group();

  constructor(scene: Scene) {
    this.#scene = scene;
    this.#scene.add(this.#playerGroup);
  }

  animate() {
    this.#players.forEach((player) => player.animate());
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "player_joined": {
        this.addPlayer(data);
        break;
      }

      case "player_left": {
        this.removePlayer(data);
        break;
      }

      case "set_player_location": {
        this.setPlayerLocation(data.playerId, data.location);
        break;
      }
    }
  }

  addPlayer(playerId: string) {
    const player = new OtherPlayer(playerId);
    this.#players.set(playerId, player);
    this.#playerGroup.add(player.mesh);
  }

  removePlayer(playerId: string) {
    const player = this.#players.get(playerId);
    if (player) {
      player.destroy();
      this.#players.delete(playerId);
    }
  }

  setPlayerLocation(
    playerId: string,
    location: [number, number, number, number, number, number, number]
  ) {
    const player = this.#players.get(playerId);
    if (player) player.setLocation(location);
  }

  destroy() {
    this.#players.forEach((player) => player.destroy());
  }
}
