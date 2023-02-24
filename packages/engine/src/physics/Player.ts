import {
  Collider,
  ColliderDesc,
  KinematicCharacterController,
  RigidBody,
  RigidBodyDesc,
  Vector3,
  World,
} from "@dimforge/rapier3d";

import {
  INPUT_ARRAY_ROUNDING,
  PLAYER_HEIGHT,
  PLAYER_RADIUS,
  POSITION_ARRAY_ROUNDING,
  ROTATION_ARRAY_ROUNDING,
  SPRINT_SPEED,
  WALK_SPEED,
} from "../constants";
import { PostMessage } from "../types";
import { COLLISION_GROUP } from "./groups";
import { FromPhysicsMessage } from "./messages";
import { PhysicsScene } from "./PhysicsScene";

const VOID_HEIGHT = -100;
const CHARACTER_OFFSET = 0.02;
const RIGID_BODY_FEET_OFFSET = PLAYER_HEIGHT / 2 + PLAYER_RADIUS + CHARACTER_OFFSET;

/**
 * Represents the player in the physics thread.
 * Reads input from the main thread, and updates the player's rigid body position.
 */
export class Player {
  #world: World;
  #scene: PhysicsScene;
  #postMessage: PostMessage<FromPhysicsMessage>;

  controller: KinematicCharacterController;
  collider: Collider;
  rigidBody: RigidBody;

  input: Int16Array | null = null;
  cameraYaw: Int16Array | null = null;
  userPosition: Int32Array | null = null;

  sprinting = false;
  shouldJump = false;
  #isGrounded = false;
  #groundedChanged = false;
  #baseYVelocity = 0;
  #ground: RigidBody | null = null;

  constructor(world: World, scene: PhysicsScene, postMessage: PostMessage<FromPhysicsMessage>) {
    this.#world = world;
    this.#scene = scene;
    this.#postMessage = postMessage;

    this.controller = this.#world.createCharacterController(CHARACTER_OFFSET);
    this.controller.enableSnapToGround(0.5);
    this.controller.enableAutostep(PLAYER_HEIGHT / 5, PLAYER_RADIUS / 2, false);
    this.controller.setSlideEnabled(true);
    this.controller.setMaxSlopeClimbAngle((60 * Math.PI) / 180);
    this.controller.setMinSlopeSlideAngle((30 * Math.PI) / 180);

    const colliderDesc = ColliderDesc.capsule(PLAYER_HEIGHT / 2, PLAYER_RADIUS);
    colliderDesc.setCollisionGroups(COLLISION_GROUP.player);

    const rigidBodyDesc = RigidBodyDesc.kinematicVelocityBased();
    this.rigidBody = this.#world.createRigidBody(rigidBodyDesc);

    this.collider = this.#world.createCollider(colliderDesc, this.rigidBody);
  }

  get isGrounded() {
    return this.#isGrounded;
  }

  set isGrounded(value: boolean) {
    if (value === this.#isGrounded) return;
    this.#isGrounded = value;
    this.#groundedChanged = true;
    this.#postMessage({ subject: "set_grounded", data: value });
  }

  jump() {
    this.shouldJump = true;
  }

  respawn() {
    const spawn = this.#scene.getSpawn();
    const position = spawn?.getWorldTranslation() ?? [0, 0, 0];
    this.rigidBody.setTranslation(
      {
        x: position[0],
        y: position[1] + RIGID_BODY_FEET_OFFSET,
        z: position[2],
      },
      true
    );
    this.rigidBody.setLinvel({ x: 0, y: 0, z: 0 }, true);
  }

  update() {
    if (!this.input || !this.cameraYaw || !this.userPosition) return;
    const delta = this.#world.timestep;

    // Read input
    const inputX = Atomics.load(this.input, 0) / INPUT_ARRAY_ROUNDING;
    const inputY = Atomics.load(this.input, 1) / INPUT_ARRAY_ROUNDING;

    // Rotate input
    const yaw = Atomics.load(this.cameraYaw, 0) / ROTATION_ARRAY_ROUNDING;
    const cos = Math.cos(yaw);
    const sin = Math.sin(yaw);
    const rotatedX = inputY * sin - inputX * cos;
    const rotatedZ = inputX * sin + inputY * cos;

    // Apply input
    const velocity = this.rigidBody.linvel();
    const speed = this.sprinting ? SPRINT_SPEED : WALK_SPEED;
    velocity.x = rotatedX * speed * 0.1;
    velocity.z = rotatedZ * speed * 0.1;

    this.isGrounded = this.controller.computedGrounded();

    if (this.#groundedChanged) {
      this.#groundedChanged = false;
      velocity.y = this.#baseYVelocity;
      this.#baseYVelocity = 0;

      // Reset ground
      this.#ground = null;
    }

    // Apply gravity
    velocity.y += this.#world.gravity.y * delta;

    if (this.isGrounded) {
      velocity.y = this.#world.gravity.y;

      // Get ground from latest collision event
      const groundCollision = this.controller.computedCollision(0);
      const groundRigidBody = groundCollision?.collider?.parent();
      if (groundRigidBody) this.#ground = groundRigidBody;
    }

    // Move player with ground
    if (this.#ground) {
      const groundVelocity = this.#ground.linvel();
      velocity.x += groundVelocity.x * delta;
      velocity.z += groundVelocity.z * delta;

      if (groundVelocity.y > 0) {
        velocity.y += groundVelocity.y;
        this.#baseYVelocity = groundVelocity.y;
      }
    }

    // Jump
    if (this.shouldJump) {
      this.shouldJump = false;
      if (this.isGrounded) {
        velocity.y = 6;
        this.#baseYVelocity = velocity.y;
      }
    }

    // Compute movement
    const inputTranslation: Vector3 = {
      x: velocity.x * delta,
      y: velocity.y * delta,
      z: velocity.z * delta,
    };

    this.controller.computeColliderMovement(this.collider, inputTranslation);
    const computedMovement = this.controller.computedMovement();

    // Apply velocity
    this.rigidBody.setLinvel(
      {
        x: computedMovement.x / delta,
        y: computedMovement.y / delta,
        z: computedMovement.z / delta,
      },
      true
    );

    // Store user position
    const pos = this.rigidBody.translation();
    Atomics.store(this.userPosition, 0, pos.x * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.userPosition, 1, (pos.y - RIGID_BODY_FEET_OFFSET) * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.userPosition, 2, pos.z * POSITION_ARRAY_ROUNDING);

    // Teleport out of void if needed
    if (pos.y < VOID_HEIGHT) this.respawn();
  }
}

export function quaternionToYaw(y: number, w: number): number {
  const yaw = Math.atan2(2 * y * w, 1 - 2 * y * y);
  return yaw;
}
