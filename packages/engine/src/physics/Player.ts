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
import { COLLISION_GROUP } from "./groups";
import { FromPhysicsMessage } from "./messages";

export class Player {
  #world: World;
  #postMessage: PostMessage<FromPhysicsMessage>;

  controller: KinematicCharacterController;
  collider: Collider;
  rigidBody: RigidBody;

  input: Int16Array;
  rotation: Int32Array;
  position: Int32Array;

  constructor(world: World, postMessage: PostMessage<FromPhysicsMessage>) {
    this.#world = world;
    this.#postMessage = postMessage;

    this.controller = this.#world.createCharacterController(0.01);
    this.controller.enableSnapToGround(0.01);
    this.controller.setSlideEnabled(true);
    this.controller.enableAutostep(PLAYER_HEIGHT / 4, PLAYER_RADIUS / 2, false);

    const colliderDesc = ColliderDesc.capsule(PLAYER_HEIGHT / 2, PLAYER_RADIUS);
    colliderDesc.setCollisionGroups(COLLISION_GROUP.player);

    const rigidBodyDesc = RigidBodyDesc.kinematicVelocityBased();
    this.rigidBody = this.#world.createRigidBody(rigidBodyDesc);

    this.collider = this.#world.createCollider(colliderDesc, this.rigidBody);

    const inputBuffer = new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2);
    this.input = new Int16Array(inputBuffer);

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

    const inputVelocity = this.rigidBody.linvel();
    inputVelocity.x = inputXRotated * 4;
    inputVelocity.z = inputYRotated * 4;

    // Only apply gravity if not grounded
    const isGrounded = this.controller.computedGrounded();
    if (isGrounded) inputVelocity.y = 0;
    else inputVelocity.y += this.#world.gravity.y * delta;

    // Compute movement
    const inputTranslation: Vector3 = {
      x: inputVelocity.x * delta,
      y: inputVelocity.y * delta,
      z: inputVelocity.z * delta,
    };

    this.controller.computeColliderMovement(this.collider, inputTranslation);
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
    Atomics.store(
      this.position,
      1,
      (pos.y - PLAYER_HEIGHT / 2 - PLAYER_RADIUS) * POSITION_ARRAY_ROUNDING
    );
    Atomics.store(this.position, 2, pos.z * POSITION_ARRAY_ROUNDING);

    // Teleport out of void if needed
    if (pos.y < -100) {
      this.rigidBody.setTranslation({ x: 0, y: 0, z: 0 }, true);
      this.rigidBody.setLinvel({ x: 0, y: 0, z: 0 }, true);
    }
  }
}
