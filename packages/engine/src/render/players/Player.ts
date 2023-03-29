import { Camera, Group } from "three";

import { POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "../../constants";
import { Avatar } from "./Avatar";

export class Player {
  readonly group = new Group();

  readonly id: number;
  readonly position: Int32Array;
  readonly rotation: Int16Array;

  #camera: Camera;
  #name: string | null = null;
  #avatar: Avatar | null = null;
  #animationsPath: string | null = null;
  #grounded = true;

  constructor(id: number, position: Int32Array, rotation: Int16Array, camera: Camera) {
    this.id = id;
    this.position = position;
    this.rotation = rotation;
    this.#camera = camera;
  }

  get name() {
    return this.#name;
  }

  set name(value: string | null) {
    if (this.#name === value) return;
    this.#name = value;
    if (this.avatar) this.avatar.name = value;
  }

  get avatar() {
    return this.#avatar;
  }

  set avatar(value: Avatar | null) {
    if (this.#avatar?.uri === value?.uri) return;

    if (value) {
      if (this.#avatar) {
        value.velocity.copy(this.#avatar.velocity);
        value.group.position.copy(this.#avatar.group.position);
        value.group.quaternion.copy(this.#avatar.group.quaternion);
        value.targetPosition.copy(this.#avatar.targetPosition);
        value.targetRotation.copy(this.#avatar.targetRotation);
      } else {
        const posX = Atomics.load(this.position, 0) / POSITION_ARRAY_ROUNDING;
        const posY = Atomics.load(this.position, 1) / POSITION_ARRAY_ROUNDING;
        const posZ = Atomics.load(this.position, 2) / POSITION_ARRAY_ROUNDING;

        const rotX = Atomics.load(this.rotation, 0) / ROTATION_ARRAY_ROUNDING;
        const rotY = Atomics.load(this.rotation, 1) / ROTATION_ARRAY_ROUNDING;
        const rotZ = Atomics.load(this.rotation, 2) / ROTATION_ARRAY_ROUNDING;
        const rotW = Atomics.load(this.rotation, 3) / ROTATION_ARRAY_ROUNDING;

        value.group.position.set(posX, posY, posZ);
        value.group.quaternion.set(rotX, rotY, rotZ, rotW);
      }

      value.animationsPath = this.animationsPath;
      value.grounded = this.grounded;
      value.name = this.name;
      this.group.add(value.group);
    }

    this.#avatar?.destroy();
    this.#avatar = value;
  }

  get grounded() {
    return this.#grounded;
  }

  set grounded(value: boolean) {
    if (this.#grounded === value) return;
    this.#grounded = value;
    if (this.avatar) this.avatar.grounded = value;
  }

  get animationsPath() {
    return this.#animationsPath;
  }

  set animationsPath(value: string | null) {
    if (this.#animationsPath === value) return;
    this.#animationsPath = value;
    if (this.avatar) this.avatar.animationsPath = value;
  }

  setAvatarUri(uri: string | null) {
    if (uri) this.avatar = new Avatar(uri, this.#camera);
    else this.avatar = null;
  }

  update(delta: number) {
    if (!this.avatar) return;

    const posX = Atomics.load(this.position, 0) / POSITION_ARRAY_ROUNDING;
    const posY = Atomics.load(this.position, 1) / POSITION_ARRAY_ROUNDING;
    const posZ = Atomics.load(this.position, 2) / POSITION_ARRAY_ROUNDING;

    const rotX = Atomics.load(this.rotation, 0) / ROTATION_ARRAY_ROUNDING;
    const rotY = Atomics.load(this.rotation, 1) / ROTATION_ARRAY_ROUNDING;
    const rotZ = Atomics.load(this.rotation, 2) / ROTATION_ARRAY_ROUNDING;
    const rotW = Atomics.load(this.rotation, 3) / ROTATION_ARRAY_ROUNDING;

    this.avatar.targetPosition.set(posX, posY, posZ);
    this.avatar.targetRotation.set(rotX, rotY, rotZ, rotW);

    this.avatar.update(delta);
  }

  destroy() {
    this.group.removeFromParent();
    this.avatar?.destroy();
  }
}
