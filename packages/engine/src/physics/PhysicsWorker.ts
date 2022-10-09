import {
  Collider,
  ColliderDesc,
  RigidBody,
  RigidBodyDesc,
  Vector,
  World,
} from "@dimforge/rapier3d";

import { EntityJSON } from "../scene";
import { PostMessage } from "../types";
import { FromPhysicsMessage, ToPhysicsMessage } from "./types";

const TERMINAL_VELOCITY = 30;
const JUMP_STRENGTH = 6;
const HZ = 60; // Physics updates per second

/*
 * Runs the physics loop.
 * Uses the Rapier physics engine.
 */
export class PhysicsWorker {
  #world = new World({ x: 0, y: -9.81, z: 0 });
  #postMessage: PostMessage<FromPhysicsMessage>;

  #playerBody: RigidBody | null = null;

  #playerPosition: Int32Array | null = null;
  #playerVelocity: Int32Array | null = null;

  #interval: NodeJS.Timer | null = null;
  #jump = false;

  #entities = new Map<string, EntityJSON>();
  #rigidBodies = new Map<string, RigidBody>();
  #colliders = new Map<string, Collider>();

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;

    // Set ready
    this.#postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToPhysicsMessage>) => {
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
      case "add_entity":
        this.#entities.set(data.entity.id, data.entity);
        this.addCollider(data.entity);
        break;
      case "remove_entity":
        this.#entities.delete(data.entityId);
        this.removeCollider(data.entityId);
        break;
      case "update_entity": {
        const entity = this.#entities.get(data.entityId);
        if (!entity) throw new Error("Entity not found");
        const updatedEntity = { ...entity, ...data.data };
        this.#entities.set(data.entityId, updatedEntity);

        this.addCollider(updatedEntity);
        break;
      }
      case "set_global_transform": {
        const rigidBody = this.#rigidBodies.get(data.entityId);
        if (!rigidBody) break;

        rigidBody.setTranslation(
          {
            x: data.position[0],
            y: data.position[1],
            z: data.position[2],
          },
          true
        );

        rigidBody.setRotation(
          {
            w: data.rotation[3],
            x: data.rotation[0],
            y: data.rotation[1],
            z: data.rotation[2],
          },
          true
        );
        break;
      }
      case "load_json": {
        // Load entities from JSON
        const entities = data.scene.entities;
        if (entities)
          entities.forEach((entity) => {
            this.#entities.set(entity.id, entity);
            this.addCollider(entity);
          });
        break;
      }
    }
  };

  start() {
    this.stop();
    // Start loop
    this.#interval = setInterval(this.#physicsLoop.bind(this), 1000 / HZ);
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
      Int32Array.BYTES_PER_ELEMENT * 3
    );
    const velocityBuffer = new SharedArrayBuffer(
      Int32Array.BYTES_PER_ELEMENT * 2
    );

    this.#playerPosition = new Int32Array(positionBuffer);
    this.#playerVelocity = new Int32Array(velocityBuffer);

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

  addCollider(entity: EntityJSON) {
    // Remove existing collider
    this.removeCollider(entity.id);

    // If entity has no collider, return
    if (!entity.collider) return;

    // Create collider description
    let colliderDesc: ColliderDesc;
    switch (entity.collider.type) {
      case "box": {
        const size = entity.collider.size;
        const halfSize: Triplet = [size[0] / 2, size[1] / 2, size[2] / 2];
        colliderDesc = ColliderDesc.cuboid(...halfSize);
        break;
      }

      case "sphere": {
        const radius = entity.collider.radius;
        colliderDesc = ColliderDesc.ball(radius);
        break;
      }

      case "cylinder": {
        const height = entity.collider.height;
        const cylinderRadius = entity.collider.radius;
        colliderDesc = ColliderDesc.cylinder(height / 2, cylinderRadius);
        break;
      }
    }

    const rigidBodyDesc = RigidBodyDesc.fixed();
    const rigidBody = this.#world.createRigidBody(rigidBodyDesc);
    const collider = this.#world.createCollider(colliderDesc, rigidBody);

    // Store reference
    this.#rigidBodies.set(entity.id, rigidBody);
    this.#colliders.set(entity.id, collider);
  }

  removeCollider(entityId: string) {
    const rigidBody = this.#rigidBodies.get(entityId);
    const collider = this.#colliders.get(entityId);

    if (rigidBody) this.#world.removeRigidBody(rigidBody);
    if (collider) this.#world.removeCollider(collider, true);

    this.#rigidBodies.delete(entityId);
    this.#colliders.delete(entityId);
  }

  #physicsLoop() {
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
        x: Atomics.load(this.#playerVelocity, 0) / 1000,
        y: limitedY,
        z: Atomics.load(this.#playerVelocity, 1) / 1000,
      };

      this.#playerBody.setLinvel(velocity, true);
    }

    // Step world
    this.#world.step();

    // Send player position to render thread
    if (this.#playerPosition && this.#playerBody) {
      const position = this.#playerBody.translation();
      Atomics.store(this.#playerPosition, 0, position.x * 1000);
      Atomics.store(this.#playerPosition, 1, position.y * 1000);
      Atomics.store(this.#playerPosition, 2, position.z * 1000);
    }
  }
}
