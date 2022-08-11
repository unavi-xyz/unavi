import { ColliderDesc, RigidBody, RigidBodyDesc, Vector, World } from "@dimforge/rapier3d";

import { FromGameMessage } from "../types";

export class GameWorker {
  #world = new World({ x: 0, y: -9.81, z: 0 });

  #playerBody: RigidBody | null = null;
  #playerPosition: Float32Array | null = null;
  #playerVelocity: Float32Array | null = null;

  constructor() {
    // Create ground
    const groundCollider = ColliderDesc.cuboid(10, 0.1, 10);
    const groundBody = this.#world.createRigidBody(RigidBodyDesc.fixed());
    this.#world.createCollider(groundCollider, groundBody);

    // Start loop
    const hz = 60; // Game updates per second
    const ms = 1000 / hz;
    setInterval(this.#gameLoop.bind(this), ms);
  }

  initPlayer() {
    // Create player body
    const playerCollider = ColliderDesc.capsule(0.5, 0.5);
    this.#playerBody = this.#world.createRigidBody(RigidBodyDesc.kinematicVelocityBased());
    this.#world.createCollider(playerCollider, this.#playerBody);
    this.#playerBody.setTranslation({ x: 0, y: 1, z: 5 }, true);

    // Create shared array buffers
    const positionBuffer = new SharedArrayBuffer(Float32Array.BYTES_PER_ELEMENT * 3);
    const velocityBuffer = new SharedArrayBuffer(Float32Array.BYTES_PER_ELEMENT * 2);
    this.#playerPosition = new Float32Array(positionBuffer);
    this.#playerVelocity = new Float32Array(velocityBuffer);

    // Send buffers to render thread
    this.#postMessage({
      subject: "player_buffers",
      data: {
        position: this.#playerPosition,
        velocity: this.#playerVelocity,
      },
    });
  }

  #gameLoop() {
    // Set player velocity
    if (this.#playerVelocity && this.#playerBody) {
      const velocity: Vector = { x: this.#playerVelocity[0], y: 0, z: this.#playerVelocity[1] };
      this.#playerBody.setLinvel(velocity, true);
    }

    // Step world
    this.#world.step();

    // Send player position to render thread
    if (this.#playerPosition && this.#playerBody) {
      const { x, y, z } = this.#playerBody.translation();
      this.#playerPosition[0] = x;
      this.#playerPosition[1] = y;
      this.#playerPosition[2] = z;
    }
  }

  #postMessage(message: FromGameMessage) {
    postMessage(message);
  }
}
