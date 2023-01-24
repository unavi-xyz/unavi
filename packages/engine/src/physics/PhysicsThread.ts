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

  constructor(postMessage: PostMessage<FromPhysicsMessage>) {
    this.player = new Player(this.world, postMessage);

    setInterval(this.update, 1000 / 60); // 60hz
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
    }
  };

  update = () => {
    if (this.controls === "player") {
      this.player.update();
    }

    this.world.step();
  };
}
