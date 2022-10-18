import {
  Capsule,
  CoefficientCombineRule,
  Collider,
  ColliderDesc,
  RigidBody,
  RigidBodyDesc,
  World,
} from "@dimforge/rapier3d";

import { PLAYER_HEIGHT, PLAYER_RADIUS } from "../constants";
import { EntityJSON } from "../scene";
import { PostMessage, Triplet } from "../types";
import {
  playerCollisionGroup,
  playerShapeCastCollisionGroup,
  staticCollisionGroup,
} from "./groups";
import { FromPhysicsMessage, ToPhysicsMessage } from "./types";

const TERMINAL_VELOCITY = 40;
const VOID_HEIGHT = -100;
const JUMP_VELOCITY = 6;
const JUMP_COOLDOWN_SECONDS = 0.2;
const HZ = 60; // Physics updates per second
const WALK_SPEED = 1;
const SPRINT_SPEED = 1.6;

const groundCollisionShape = new Capsule(
  PLAYER_HEIGHT / 2,
  PLAYER_RADIUS - 0.05 // Slightly smaller radius to avoid sticking to walls
);
const PLAYER_FRICTION = 0.5;

/*
 * Runs the physics loop.
 * Uses the Rapier physics engine.
 */
export class PhysicsWorker {
  #world = new World({ x: 0, y: -9.81, z: 0 });
  #postMessage: PostMessage<FromPhysicsMessage>;

  #playerBody: RigidBody | null = null;
  #playerCollider: Collider | null = null;

  #playerPosition: Int32Array | null = null;
  #playerVelocity: Int32Array | null = null;

  #interval: NodeJS.Timer | null = null;
  #jump = false;
  #lastJumpTime = 0;
  #isGrounded = false;
  #isSprinting = false;

  #entities = new Map<string, EntityJSON>();
  #rigidBodies = new Map<string, RigidBody>();
  #colliders = new Map<string, Collider>();
  #spawn: Triplet = [0, 0, 0];

  constructor(postMessage: PostMessage) {
    this.#postMessage = postMessage;

    // Set ready
    this.#postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToPhysicsMessage>) => {
    const { subject, data } = event.data;
    switch (subject) {
      case "start": {
        this.start();
        break;
      }

      case "stop": {
        this.stop();
        break;
      }

      case "init_player": {
        this.initPlayer();
        break;
      }

      case "jumping": {
        this.setJumping(data);
        break;
      }

      case "add_entity": {
        this.#entities.set(data.entity.id, data.entity);
        this.addCollider(data.entity);
        break;
      }

      case "remove_entity": {
        this.#entities.delete(data.entityId);
        this.removeCollider(data.entityId);
        break;
      }

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
        // Save spawn
        if (data.scene.spawn) {
          this.#spawn = data.scene.spawn;
          this.#spawn[1] += PLAYER_HEIGHT / 2;

          // Set player position
          if (this.#playerBody) {
            this.#playerBody.setTranslation(
              {
                x: this.#spawn[0],
                y: this.#spawn[1],
                z: this.#spawn[2],
              },
              true
            );
            this.#playerBody.setLinvel({ x: 0, y: 0, z: 0 }, true);
          }
        }

        // Load entities from JSON
        const entities = data.scene.entities;
        if (entities)
          entities.forEach((entity) => {
            this.#entities.set(entity.id, entity);
            this.addCollider(entity);
          });
        break;
      }

      case "sprinting": {
        this.#isSprinting = data;
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
    const playerColliderDesc = ColliderDesc.capsule(
      PLAYER_HEIGHT / 2,
      PLAYER_RADIUS
    );
    playerColliderDesc.setCollisionGroups(playerCollisionGroup);
    playerColliderDesc.setFriction(PLAYER_FRICTION);
    playerColliderDesc.setFrictionCombineRule(CoefficientCombineRule.Min);

    const playerBodyDesc = RigidBodyDesc.dynamic()
      .setTranslation(...this.#spawn)
      .lockRotations();

    this.#playerBody = this.#world.createRigidBody(playerBodyDesc);
    this.#playerCollider = this.#world.createCollider(
      playerColliderDesc,
      this.#playerBody
    );

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

    colliderDesc.setCollisionGroups(staticCollisionGroup);

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
    if (this.#playerBody && this.#playerCollider) {
      const playerPosition = this.#playerBody.translation();
      const playerRotation = this.#playerBody.rotation();

      // If player is in the void, reset position
      if (playerPosition.y < VOID_HEIGHT) {
        this.#playerBody.setTranslation(
          {
            x: this.#spawn[0],
            y: this.#spawn[1],
            z: this.#spawn[2],
          },
          true
        );
        this.#playerBody.setLinvel({ x: 0, y: 0, z: 0 }, true);
      }

      const velocity = this.#playerBody.linvel();

      const groundedCollision = this.#world.castShape(
        playerPosition,
        playerRotation,
        this.#world.gravity,
        groundCollisionShape,
        this.#world.timestep,
        playerShapeCastCollisionGroup,
        playerShapeCastCollisionGroup
      );

      const isGrounded = Boolean(groundedCollision);

      if (isGrounded !== this.#isGrounded) {
        this.#isGrounded = isGrounded;
        this.#postMessage({
          subject: "player_falling",
          data: !isGrounded,
        });
      }

      // Set player friction to 0 in air
      const friction = this.#playerCollider.friction();

      if (isGrounded && friction !== PLAYER_FRICTION) {
        this.#playerCollider.setFriction(PLAYER_FRICTION);
      } else if (!isGrounded && friction !== 0) {
        this.#playerCollider.setFriction(0);
      }

      // Jumping
      const tryJump = this.#jump && isGrounded;
      if (this.#jump) this.#jump = false;

      const time = performance.now();
      const deltaJump = time - this.#lastJumpTime;
      const jumpCooldownActive = deltaJump < JUMP_COOLDOWN_SECONDS * 1000;

      const doJump = tryJump && !jumpCooldownActive;
      if (doJump) {
        velocity.y = JUMP_VELOCITY;
        this.#lastJumpTime = time;
      }

      // Apply input
      if (this.#playerVelocity) {
        const speed = this.#isSprinting ? SPRINT_SPEED : WALK_SPEED;
        velocity.x = (Atomics.load(this.#playerVelocity, 0) / 1000) * speed;
        velocity.z = (Atomics.load(this.#playerVelocity, 1) / 1000) * speed;
      }

      // Cap y velocity to prevent going too fast
      velocity.y = Math.max(
        Math.min(velocity.y, TERMINAL_VELOCITY),
        -TERMINAL_VELOCITY
      );

      this.#playerBody.setLinvel(velocity, true);
    }

    // Step world
    this.#world.step();

    // Send player position to render thread
    if (this.#playerPosition && this.#playerBody) {
      const position = this.#playerBody.translation();
      const playerFeet = position.y - PLAYER_HEIGHT / 2 - PLAYER_RADIUS;
      Atomics.store(this.#playerPosition, 0, position.x * 1000);
      Atomics.store(this.#playerPosition, 1, playerFeet * 1000);
      Atomics.store(this.#playerPosition, 2, position.z * 1000);
    }
  }
}
