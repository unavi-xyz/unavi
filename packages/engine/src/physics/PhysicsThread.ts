import { World } from "@dimforge/rapier3d";

import { DEFAULT_CONTROLS } from "../constants";
import { ControlsType } from "../Engine";
import { isSceneMessage } from "../scene/messages";
import { PostMessage } from "../types";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";
import { PhysicsScene } from "./PhysicsScene";
import { Player } from "./Player";

export class PhysicsThread {
  controls: ControlsType = DEFAULT_CONTROLS;

  world = new World({ x: 0, y: -9.81, z: 0 });
  scene = new PhysicsScene(this.world);
  player: Player;

  #interval: NodeJS.Timeout;

  constructor(postMessage: PostMessage<FromPhysicsMessage>) {
    this.player = new Player(this.world, postMessage);

    this.#interval = setInterval(this.update, 1000 / 60); // 60hz

    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToPhysicsMessage>) => {
    if (isSceneMessage(event.data)) {
      this.scene.onmessage(event.data);
      return;
    }

    const { subject, data } = event.data;

    switch (subject) {
      case "set_controls": {
        this.controls = data;
        break;
      }

      case "set_sprinting": {
        this.player.sprinting = data;
        break;
      }

      case "destroy": {
        clearInterval(this.#interval);
        break;
      }
    }
  };

  update = () => {
    if (this.controls === "player") {
      this.player.update();
    }

    this.world.step();
  };
}
