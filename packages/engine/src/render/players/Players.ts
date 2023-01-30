import { Camera, Group } from "three";

import { ToRenderMessage } from "../messages";
import { Player } from "./Player";

export class Players {
  readonly group = new Group();

  #store = new Map<number, Player>();
  #defaultAvatar: string | null = null;
  #animationsPath: string | null = null;
  #camera: Camera;

  constructor(camera: Camera) {
    this.#camera = camera;
  }

  onmessage({ subject, data }: ToRenderMessage) {
    switch (subject) {
      case "add_player": {
        const player = new Player(data.id, data.position, data.rotation, this.#camera);
        player.animationsPath = this.#animationsPath;
        this.group.add(player.group);

        setTimeout(() => {
          if (!player.avatar) player.setAvatarUri(this.#defaultAvatar);
        }, 10);

        this.#store.set(data.id, player);
        break;
      }

      case "remove_player": {
        const player = this.#store.get(data);
        if (player) player.destroy();
        this.#store.delete(data);
        break;
      }

      case "set_player_name": {
        const player = this.#store.get(data.playerId);
        if (player) player.name = data.name;
        break;
      }

      case "set_player_avatar": {
        const player = this.#store.get(data.playerId);
        if (player) player.setAvatarUri(data.uri);
        break;
      }

      case "set_player_grounded": {
        const player = this.#store.get(data.playerId);
        if (player) player.grounded = data.grounded;
        break;
      }

      case "set_default_avatar": {
        this.#defaultAvatar = data;
        this.#store.forEach((player) => {
          if (!player.avatar) player.setAvatarUri(data);
        });
        break;
      }

      case "set_animations_path": {
        this.#animationsPath = data;
        this.#store.forEach((player) => {
          if (player.avatar) player.animationsPath = data;
        });
      }
    }
  }

  update(delta: number) {
    this.#store.forEach((player) => player.update(delta));
  }
}
