import { Euler, PerspectiveCamera, Quaternion, Vector3 } from "three";

import { POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "../../constants";
import { ToRenderMessage } from "../messages";

export class PlayerControls {
  #camera: PerspectiveCamera;

  positionArray: Int32Array | null = null;
  rotationArray: Int32Array | null = null;

  #targetCameraRotation = new Quaternion();
  #euler = new Euler(0, 0, 0, "YXZ");
  #vec3 = new Vector3();

  #delta = 0;

  constructor(camera: PerspectiveCamera) {
    this.#camera = camera;
  }

  onmessage({ subject, data }: ToRenderMessage) {
    switch (subject) {
      case "set_player_arrays": {
        this.positionArray = data.position;
        this.rotationArray = data.rotation;
        break;
      }

      case "mousemove": {
        this.#euler.setFromQuaternion(this.#targetCameraRotation);

        this.#euler.y -= (data.x * this.#delta) / 4;
        this.#euler.x -= (data.y * this.#delta) / 4;

        const minPolarAngle = 0;
        const maxPolarAngle = Math.PI;

        this.#euler.x = Math.max(
          Math.PI / 2 - maxPolarAngle,
          Math.min(Math.PI / 2 - minPolarAngle, this.#euler.x)
        );

        this.#targetCameraRotation.setFromEuler(this.#euler);
        break;
      }
    }
  }

  update(delta: number) {
    this.#delta = delta;

    if (!this.positionArray || !this.rotationArray) return;

    // Move camera to player position
    const x = Atomics.load(this.positionArray, 0) / POSITION_ARRAY_ROUNDING;
    const y = Atomics.load(this.positionArray, 1) / POSITION_ARRAY_ROUNDING;
    const z = Atomics.load(this.positionArray, 2) / POSITION_ARRAY_ROUNDING;
    this.#camera.position.set(x, y, z);

    // Rotate camera
    const K = 1 - Math.pow(10e-24, delta);
    this.#camera.quaternion.slerp(this.#targetCameraRotation, K);

    // Store camera rotation
    const direction = this.#camera.getWorldDirection(this.#vec3);
    const angle = Math.atan2(direction.x, direction.z);
    Atomics.store(this.rotationArray, 0, angle * ROTATION_ARRAY_ROUNDING);
  }
}
