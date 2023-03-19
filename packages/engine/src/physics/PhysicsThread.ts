import { World } from "@dimforge/rapier3d";

import { DEFAULT_CONTROLS, DEFAULT_VISUALS } from "../constants";
import { ControlsType, PostMessage } from "../types";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";
import { Player } from "./Player";
import { PhysicsScene } from "./scene/PhysicsScene";

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

  #visuals = DEFAULT_VISUALS;
  #visualsFrame = 0;
  #debugVertices = new Float32Array(0);
  #debugColors = new Float32Array(0);
  #debugInterval = 30;

  #interval: NodeJS.Timeout | null = null;
  #postMessage: PostMessage<FromPhysicsMessage>;

  constructor(postMessage: PostMessage<FromPhysicsMessage>) {
    this.#postMessage = postMessage;

    this.player = new Player(this.world, this.scene, postMessage);

    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToPhysicsMessage>) => {
    this.scene.onmessage(event.data);

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
        this.player.enabled = data === "player";
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

      case "toggle_visuals": {
        this.#visuals = data;
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

    if (this.#visuals) {
      // Only update the debug buffers every 30 frames
      if (this.#visualsFrame++ % this.#debugInterval === 0) {
        const buffers = this.world.debugRender();

        // Change the size of the shared buffers if needed
        let didChange = false;

        if (buffers.vertices.length !== this.#debugVertices.length) {
          const newBuffer = new SharedArrayBuffer(
            buffers.vertices.length * Float32Array.BYTES_PER_ELEMENT
          );
          this.#debugVertices = new Float32Array(newBuffer);
          didChange = true;
        }

        if (buffers.colors.length !== this.#debugColors.length) {
          const newBuffer = new SharedArrayBuffer(
            buffers.colors.length * Float32Array.BYTES_PER_ELEMENT
          );
          this.#debugColors = new Float32Array(newBuffer);
          didChange = true;
        }

        if (didChange) {
          this.#postMessage({
            subject: "set_debug_buffers",
            data: { vertices: this.#debugVertices, colors: this.#debugColors },
          });
        }

        // Copy data to shared buffers
        this.#debugVertices.set(buffers.vertices);
        this.#debugColors.set(buffers.colors);

        // Update interval based on the number of debug vertices
        this.#debugInterval = Math.max(8, Math.floor(this.#debugVertices.length / 40000));
      }
    }
  };
}
