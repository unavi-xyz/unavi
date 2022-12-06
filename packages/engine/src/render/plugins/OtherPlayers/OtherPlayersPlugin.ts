import { Camera, Group, Scene, WebGLRenderer } from "three";

import { PostMessage } from "../../../types";
import { FromRenderMessage, ToRenderMessage } from "../../types";
import { RenderPlugin } from "../types";
import { PlayerAvatar } from "./PlayerAvatar";

export class OtherPlayersPlugin implements RenderPlugin {
  #scene: Scene;
  #postMessage: PostMessage<FromRenderMessage>;
  #avatarPath?: string;
  #avatarAnimationsPath?: string;

  #renderer: WebGLRenderer;
  #camera: Camera;

  #players = new Map<number, PlayerAvatar>();
  #playerGroup = new Group();

  constructor(
    scene: Scene,
    postMessage: PostMessage<FromRenderMessage>,
    renderer: WebGLRenderer,
    camera: Camera,
    avatarPath?: string,
    avatarAnimationsPath?: string
  ) {
    this.#scene = scene;
    this.#postMessage = postMessage;
    this.#avatarPath = avatarPath;
    this.#avatarAnimationsPath = avatarAnimationsPath;

    this.#renderer = renderer;
    this.#camera = camera;

    this.#scene.add(this.#playerGroup);
  }

  update(delta: number) {
    this.#players.forEach((player) => player.update(delta));
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "player_joined": {
        this.addPlayer(data.playerId, data.avatar);
        break;
      }

      case "player_left": {
        this.removePlayer(data);
        break;
      }

      case "player_location": {
        this.setPlayerLocation(data);
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
        break;
      }

      case "clear_players": {
        this.#players.forEach((player) => this.removePlayer(player.playerId));
        break;
      }

      case "player_name": {
        const player = this.#players.get(data.playerId);
        if (player) player.setName(data.name);
        break;
      }
    }
  }

  addPlayer(playerId: number, avatar: string | null) {
    const player = new PlayerAvatar(
      playerId,
      avatar,
      this.#postMessage,
      this.#camera,
      this.#renderer,
      this.#avatarPath,
      this.#avatarAnimationsPath
    );

    this.#players.set(playerId, player);
    this.#playerGroup.add(player.group);
  }

  removePlayer(playerId: number) {
    const player = this.#players.get(playerId);
    if (player) {
      this.#playerGroup.remove(player.group);
      player.destroy();
      this.#players.delete(playerId);
    }
  }

  setPlayerLocation(buffer: ArrayBuffer) {
    const view = new DataView(buffer);

    const playerId = view.getUint8(0);

    const posX = view.getInt32(1, true) / 1000;
    const posY = view.getInt32(5, true) / 1000;
    const posZ = view.getInt32(9, true) / 1000;

    const rotX = view.getInt16(13, true) / 1000;
    const rotY = view.getInt16(15, true) / 1000;
    const rotZ = view.getInt16(17, true) / 1000;
    const rotW = view.getInt16(19, true) / 1000;

    const player = this.#players.get(playerId);
    if (player) {
      player.setPosition(posX, posY, posZ);
      player.setRotation(rotX, rotY, rotZ, rotW);
    }
  }

  destroy() {
    this.#players.forEach((player) => player.destroy());
  }
}
