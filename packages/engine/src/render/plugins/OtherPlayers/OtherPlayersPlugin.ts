import { Group, Scene } from "three";

import { ToRenderMessage } from "../../types";
import { OtherPlayer } from "./OtherPlayer";

export class OtherPlayersPlugin {
  #scene: Scene;
  #avatarPath?: string;
  #avatarAnimationsPath?: string;

  #players = new Map<string, OtherPlayer>();
  #playerGroup = new Group();

  constructor(
    scene: Scene,
    avatarPath?: string,
    avatarAnimationsPath?: string
  ) {
    this.#scene = scene;
    this.#avatarPath = avatarPath;
    this.#avatarAnimationsPath = avatarAnimationsPath;

    this.#scene.add(this.#playerGroup);
  }

  animate(delta: number) {
    this.#players.forEach((player) => player.animate(delta));
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

      case "set_player_falling_state": {
        const player = this.#players.get(data.playerId);
        if (player) player.isFalling = data.isFalling;
        break;
      }

      case "set_player_avatar": {
        const player = this.#players.get(data.playerId);
        if (player) player.setAvatar(data.avatar);
      }
    }
  }

  addPlayer(playerId: string) {
    const player = new OtherPlayer(
      playerId,
      this.#avatarPath,
      this.#avatarAnimationsPath
    );

    this.#players.set(playerId, player);
    this.#playerGroup.add(player.group);
  }

  removePlayer(playerId: string) {
    const player = this.#players.get(playerId);
    if (player) {
      this.#playerGroup.remove(player.group);
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
