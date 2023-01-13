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
} from "../constants";
import { PostMessage } from "../types";
import { FromPhysicsMessage } from "./messages";

export class Player {
  #world: World;
  #postMessage: PostMessage<FromPhysicsMessage>;

  controller: KinematicCharacterController;
  collider: Collider;
  rigidBody: RigidBody;

  input: Int32Array;
  rotation: Int32Array;
  position: Int32Array;

  constructor(world: World, postMessage: PostMessage<FromPhysicsMessage>) {
    this.#world = world;
    this.#postMessage = postMessage;

    this.controller = this.#world.createCharacterController(0.01);

    const colliderDesc = ColliderDesc.capsule(PLAYER_HEIGHT / 2, PLAYER_RADIUS);
    this.collider = this.#world.createCollider(colliderDesc);

    const rigidBodyDesc = RigidBodyDesc.kinematicVelocityBased();
    this.rigidBody = this.#world.createRigidBody(rigidBodyDesc);

    const inputBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2);
    this.input = new Int32Array(inputBuffer);

    const rotationBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 1);
    this.rotation = new Int32Array(rotationBuffer);

    const positionBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 3);
    this.position = new Int32Array(positionBuffer);

    this.#postMessage({
      subject: "set_player_arrays",
      data: {
        input: this.input,
        rotation: this.rotation,
        position: this.position,
      },
    });
  }

  update() {
    const delta = this.#world.timestep;

    // Read input
    const inputX = Atomics.load(this.input, 0) / INPUT_ARRAY_ROUNDING;
    const inputY = Atomics.load(this.input, 1) / INPUT_ARRAY_ROUNDING;

    // Rotate input
    const rotation = Atomics.load(this.rotation, 0) / ROTATION_ARRAY_ROUNDING;
    const cos = Math.cos(rotation);
    const sin = Math.sin(rotation);
    const inputXRotated = inputY * sin - inputX * cos;
    const inputYRotated = inputX * sin + inputY * cos;

    const inputVelocity: Vector3 = { x: inputXRotated, y: 0, z: inputYRotated };

    // Apply speed
    inputVelocity.x *= 0.1;
    inputVelocity.z *= 0.1;

    // Compute movement
    this.controller.computeColliderMovement(this.collider, inputVelocity);
    const computedMovement = this.controller.computedMovement();

    const computedVelocity: Vector3 = {
      x: computedMovement.x / delta,
      y: computedMovement.y / delta,
      z: computedMovement.z / delta,
    };

    // Apply velocity
    this.rigidBody.setLinvel(computedVelocity, true);

    // Store position
    const pos = this.rigidBody.translation();
    Atomics.store(this.position, 0, pos.x * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.position, 1, pos.y * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.position, 2, pos.z * POSITION_ARRAY_ROUNDING);
  }
}
