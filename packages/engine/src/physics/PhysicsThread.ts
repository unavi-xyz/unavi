import { World } from "@dimforge/rapier3d";

import { DEFAULT_CONTROLS } from "../constants";
import { ControlsType } from "../Engine";
import { isSceneMessage } from "../scene/messages";
import { PostMessage } from "../types";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";
import { PhysicsScene } from "./PhysicsScene";
import { Player } from "./Player";

const PHYSICS_LOOP_HZ = 60;

/**
 * The physics thread is responsible for running the physics loop.
 * The loop is run at a fixed rate of 60Hz.
 */
export class PhysicsThread {
  controls: ControlsType = DEFAULT_CONTROLS;

  world = new World({ x: 0, y: -9.81, z: 0 });
  scene = new PhysicsScene(this.world);
  player: Player;

  #interval: NodeJS.Timeout | null = null;

  constructor(postMessage: PostMessage<FromPhysicsMessage>) {
    this.player = new Player(this.world, this.scene, postMessage);

    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToPhysicsMessage>) => {
    if (isSceneMessage(event.data)) {
      this.scene.onmessage(event.data);
      return;
    }

    const { subject, data } = event.data;

    switch (subject) {
      case "set_user_arrays": {
        this.player.input = data.input;
        this.player.userPosition = data.userPosition;
        this.player.cameraYaw = data.cameraYaw;
        break;
      }

      case "set_controls": {
        this.controls = data;
        break;
      }

      case "set_sprinting": {
        this.player.sprinting = data;
        break;
      }

      case "jump": {
        this.player.jump();
        break;
      }

      case "start": {
        this.start();
        break;
      }

      case "stop": {
        this.stop();
        break;
      }

      case "destroy": {
        this.stop();
        break;
      }

      case "respawn": {
        this.player.respawn();
        break;
      }
    }
  };

  start() {
    this.stop();
    this.world.timestep = 1 / PHYSICS_LOOP_HZ;
    this.#interval = setInterval(this.update, 1000 / PHYSICS_LOOP_HZ);
  }

  stop() {
    if (this.#interval) clearInterval(this.#interval);
  }

  update = () => {
    if (this.controls === "player") {
      this.player.update();
    }

    this.world.step();
  };
}
