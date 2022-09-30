import {
  Collider,
  ColliderDesc,
  RigidBody,
  RigidBodyDesc,
  Vector,
  World,
} from "@dimforge/rapier3d";
import { Subscription } from "rxjs";

import { Entity, SceneMessage } from "../scene";
import { PostMessage } from "../types";
import { GameScene } from "./GameScene";
import { FromGameMessage, ToGameMessage } from "./types";

const TERMINAL_VELOCITY = 30;
const JUMP_STRENGTH = 6;
const HZ = 60; // Game updates per second

/*
 * Runs the physics loop.
 * Uses the Rapier physics engine.
 */
export class GameWorker {
  #scene: GameScene;

  #world = new World({ x: 0, y: -9.81, z: 0 });
  #postMessage: PostMessage<FromGameMessage>;

  #playerBody: RigidBody | null = null;
  #playerPosition: Float32Array | null = null;
  #playerVelocity: Float32Array | null = null;

  #interval: NodeJS.Timer | null = null;
  #lastUpdate = 0;
  #jump = false;

  #colliders = new Map<string, Collider>();
  subscriptions = new Map<string, Subscription>();

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;
    this.#scene = new GameScene(postMessage);

    // Create ground
    const groundColliderDesc = ColliderDesc.cuboid(10, 0.5, 10);
    const groundBodyDesc = RigidBodyDesc.fixed().setTranslation(0, -1, 0);
    const groundBody = this.#world.createRigidBody(groundBodyDesc);
    this.#world.createCollider(groundColliderDesc, groundBody);

    // Subscriptions
    this.#scene.entities$.subscribe({
      next: (entities) => {
        // Add new entities
        Object.values(entities).forEach((entity) => {
          if (!this.subscriptions.has(entity.id)) {
            const subscription = entity.collider$.subscribe({
              next: () => {
                this.addCollider(entity);
              },
            });

            this.subscriptions.set(entity.id, subscription);
          }
        });

        // Remove old entities
        this.subscriptions.forEach((subscription, id) => {
          if (!entities[id]) {
            subscription.unsubscribe();
            this.subscriptions.delete(id);
          }
        });
      },
    });

    // Set ready
    this.#postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToGameMessage>) => {
    this.#scene.onmessage(event as MessageEvent<SceneMessage>);

    const { subject, data } = event.data;
    switch (subject) {
      case "start":
        this.start();
        break;
      case "stop":
        this.stop();
        break;
      case "init_player":
        this.initPlayer();
        break;
      case "jumping":
        this.setJumping(data);
        break;
    }
  };

  start() {
    this.stop();
    // Start loop
    this.#interval = setInterval(this.#gameLoop.bind(this), 1000 / HZ);
  }

  stop() {
    // Stop loop
    if (this.#interval) {
      clearInterval(this.#interval);
      this.#interval = null;
    }
  }

  initPlayer() {
    // Create player body
    const playerColliderDesc = ColliderDesc.capsule(0.9, 0.2);
    const playerBodyDesc = RigidBodyDesc.dynamic()
      .setTranslation(0, 2, 5)
      .lockRotations();
    this.#playerBody = this.#world.createRigidBody(playerBodyDesc);
    this.#world.createCollider(playerColliderDesc, this.#playerBody);

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

  addCollider(entity: Entity) {
    // Remove existing collider
    const previousCollider = this.#colliders.get(entity.id);
    if (previousCollider) {
      this.#colliders.delete(entity.id);
      this.#world.removeCollider(previousCollider, true);
      const rigidBody = previousCollider.parent();
      if (rigidBody) this.#world.removeRigidBody(rigidBody);
    }

    if (!entity.collider) return;

    // Create collider description
    let colliderDesc: ColliderDesc;
    switch (entity.collider.type) {
      case "Box": {
        const size = entity.collider.size;
        colliderDesc = ColliderDesc.cuboid(...size);
        break;
      }
      case "Sphere": {
        const radius = entity.collider.radius;
        colliderDesc = ColliderDesc.ball(radius);
        break;
      }
      case "Cylinder": {
        const height = entity.collider.height;
        const cylinderRadius = entity.collider.radius;
        colliderDesc = ColliderDesc.cylinder(height / 2, cylinderRadius);
        break;
      }
    }

    const rigidBodyDesc = RigidBodyDesc.fixed();
    const rigidBody = this.#world.createRigidBody(rigidBodyDesc);
    const collider = this.#world.createCollider(colliderDesc, rigidBody);

    entity.globalPosition$.subscribe({
      next: (globalPosition) => {
        rigidBody.setTranslation(
          {
            x: globalPosition[0],
            y: globalPosition[1],
            z: globalPosition[2],
          },
          true
        );
      },
    });

    entity.globalQuaternion$.subscribe({
      next: (globalQuaternion) => {
        rigidBody.setRotation(
          {
            w: globalQuaternion[3],
            x: globalQuaternion[0],
            y: globalQuaternion[1],
            z: globalQuaternion[2],
          },
          true
        );
      },
    });

    // Store reference
    this.#colliders.set(entity.id, collider);
  }

  #gameLoop() {
    this.#lastUpdate = performance.now();

    // Jumping
    const jumpVelocity = this.#jump ? JUMP_STRENGTH : 0;
    if (this.#jump) this.#jump = false;

    // Set player velocity
    if (this.#playerVelocity && this.#playerBody) {
      const velocityY = this.#playerBody.linvel().y + jumpVelocity;
      const limitedY = Math.max(
        Math.min(velocityY, TERMINAL_VELOCITY),
        -TERMINAL_VELOCITY
      );

      const velocity: Vector = {
        x: this.#playerVelocity[0],
        y: limitedY,
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
