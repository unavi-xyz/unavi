import {
  Collider,
  ColliderDesc,
  RigidBody,
  RigidBodyDesc,
  Vector,
  World,
} from "@dimforge/rapier3d";

import { OMICollider, PostMessage, Triplet } from "../types";
import { FromGameMessage, ToGameMessage } from "./types";

const TERMINAL_VELOCITY = 50;
const JUMP_STRENGTH = 6;
const HZ = 60; // Game updates per second

export class GameWorker {
  #world = new World({ x: 0, y: -9.81, z: 0 });
  #postMessage: PostMessage<FromGameMessage>;

  #playerBody: RigidBody | null = null;
  #playerPosition: Float32Array | null = null;
  #playerVelocity: Float32Array | null = null;

  #lastUpdate = 0;
  #velocityY = 0;
  #jump = false;

  #colliders = new Map<string, Collider>();

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;

    // Create ground
    const groundCollider = ColliderDesc.cuboid(10, 0.1, 10);
    const groundBody = this.#world.createRigidBody(RigidBodyDesc.fixed());
    this.#world.createCollider(groundCollider, groundBody);

    // Start loop
    setInterval(this.#gameLoop.bind(this), 1000 / HZ);

    // Set ready
    this.#postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToGameMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "init_player":
        this.initPlayer();
        break;
      case "jumping":
        this.setJumping(data);
        break;
      case "set_physics":
        this.setPhysics(data);
        break;
    }
  };

  initPlayer() {
    // Create player body
    const playerCollider = ColliderDesc.capsule(0.5, 0.5);
    this.#playerBody = this.#world.createRigidBody(
      RigidBodyDesc.kinematicVelocityBased()
    );
    this.#world.createCollider(playerCollider, this.#playerBody);
    this.#playerBody.setTranslation({ x: 0, y: 1, z: 5 }, true);

    // Create shared array buffers
    const positionBuffer = new SharedArrayBuffer(
      Float32Array.BYTES_PER_ELEMENT * 3
    );
    const velocityBuffer = new SharedArrayBuffer(
      Float32Array.BYTES_PER_ELEMENT * 2
    );
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

  setJumping(jump: boolean) {
    this.#jump = jump;
  }

  setPhysics({
    entityId,
    collider,
    position,
    rotation,
  }: {
    entityId: string;
    collider: OMICollider | null;
    position: Triplet;
    rotation: Triplet;
  }) {
    // Remove existing collider
    const previousCollider = this.#colliders.get(entityId);
    if (previousCollider) {
      this.#colliders.delete(entityId);
      this.#world.removeCollider(previousCollider, true);
      const rigidBody = previousCollider.parent();
      if (rigidBody) this.#world.removeRigidBody(rigidBody);
    }

    if (!collider) return;

    // Create collider description
    let colliderDesc: ColliderDesc;
    switch (collider.type) {
      case "box":
        const extents = collider?.extents ?? [1, 1, 1];
        colliderDesc = ColliderDesc.cuboid(...extents);
        break;
      case "sphere":
        const radius = collider?.radius ?? 1;
        colliderDesc = ColliderDesc.ball(radius);
        break;
      case "cylinder":
        const height = collider?.height ?? 1;
        const cylinderRadius = collider?.radius ?? 1;
        colliderDesc = ColliderDesc.cylinder(height / 2, cylinderRadius);
        break;
      default:
        throw new Error(`Collider type ${collider.type} not supported`);
    }

    colliderDesc.setTranslation(...position);
    // colliderDesc.setRotation(...rotation);

    // Create rigid body
    const newBody = this.#world.createRigidBody(RigidBodyDesc.fixed());

    // Add collider to world
    const newCollider = this.#world.createCollider(colliderDesc, newBody);

    // Store reference
    this.#colliders.set(entityId, newCollider);
  }

  #gameLoop() {
    const delta = (performance.now() - this.#lastUpdate) / 1000;
    this.#lastUpdate = performance.now();

    // Gravity
    this.#velocityY -= 9.81 * delta;
    this.#velocityY = Math.max(this.#velocityY, -TERMINAL_VELOCITY);
    this.#velocityY = Math.min(this.#velocityY, TERMINAL_VELOCITY);

    // Jumping
    if (this.#jump) {
      this.#velocityY = JUMP_STRENGTH;
      this.#jump = false;
    }

    // Ground collision
    if (this.#playerBody) {
      const { x, y, z } = this.#playerBody.translation();
      if (y < 0) {
        this.#playerBody.setTranslation({ x, y: 0, z }, true);
        this.#velocityY = 0;
      }
    }

    // Set player velocity
    if (this.#playerVelocity && this.#playerBody) {
      const velocity: Vector = {
        x: this.#playerVelocity[0],
        y: this.#velocityY,
        z: this.#playerVelocity[1],
      };

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
}
