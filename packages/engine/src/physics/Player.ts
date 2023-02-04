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

const CHARACTER_OFFSET = 0.01;
const RIGID_BODY_FEET_OFFSET = PLAYER_HEIGHT / 2 + PLAYER_RADIUS + CHARACTER_OFFSET;

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

  velocity: Vector3 = { x: 0, y: 0, z: 0 };
  sprinting = false;
  shouldJump = false;
  #isGrounded = false;

  constructor(world: World, scene: PhysicsScene, postMessage: PostMessage<FromPhysicsMessage>) {
    this.#world = world;
    this.#scene = scene;
    this.#postMessage = postMessage;

    this.controller = this.#world.createCharacterController(CHARACTER_OFFSET);
    this.controller.enableSnapToGround(0.05);
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
    const delta = this.#world.timestep;

    if (!this.input || !this.cameraYaw || !this.userPosition) return;

    // Read input
    const inputX = Atomics.load(this.input, 0) / INPUT_ARRAY_ROUNDING;
    const inputY = Atomics.load(this.input, 1) / INPUT_ARRAY_ROUNDING;

    // Rotate input
    const yaw = Atomics.load(this.cameraYaw, 0) / ROTATION_ARRAY_ROUNDING;
    const cos = Math.cos(yaw);
    const sin = Math.sin(yaw);
    const rotatedX = inputY * sin - inputX * cos;
    const rotatedZ = inputX * sin + inputY * cos;

    // Calculate velocity
    const speed = this.sprinting ? SPRINT_SPEED : WALK_SPEED;

    const inputVelocity = this.rigidBody.linvel();
    inputVelocity.x = (rotatedX * speed) / 10;
    inputVelocity.z = (rotatedZ * speed) / 10;

    // Only apply gravity if not grounded
    this.isGrounded = this.controller.computedGrounded();
    if (this.isGrounded) inputVelocity.y = 0;
    else inputVelocity.y += this.#world.gravity.y * delta;

    // Lerp input velocity
    const K = 1 - Math.pow(10e-16, delta);
    inputVelocity.x = inputVelocity.x * K + this.velocity.x * (1 - K);
    inputVelocity.z = inputVelocity.z * K + this.velocity.z * (1 - K);

    // Jump
    if (this.shouldJump) {
      this.shouldJump = false;
      if (this.isGrounded) inputVelocity.y = 5;
    }

    // Compute movement
    const inputTranslation: Vector3 = {
      x: inputVelocity.x * delta,
      y: inputVelocity.y * delta,
      z: inputVelocity.z * delta,
    };

    this.controller.computeColliderMovement(this.collider, inputTranslation);
    const computedMovement = this.controller.computedMovement();

    this.velocity = {
      x: computedMovement.x / delta,
      y: computedMovement.y / delta,
      z: computedMovement.z / delta,
    };

    // Apply velocity
    this.rigidBody.setLinvel(this.velocity, true);

    // Store user position
    const pos = this.rigidBody.translation();
    Atomics.store(this.userPosition, 0, pos.x * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.userPosition, 1, (pos.y - RIGID_BODY_FEET_OFFSET) * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.userPosition, 2, pos.z * POSITION_ARRAY_ROUNDING);

    // Teleport out of void if needed
    if (pos.y < -100) this.respawn();
  }
}

export function quaternionToYaw(y: number, w: number): number {
  const yaw = Math.atan2(2 * y * w, 1 - 2 * y * y);
  return yaw;
}
