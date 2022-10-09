import { Euler, MathUtils, PerspectiveCamera, Vector2, Vector3 } from "three";

import { PostMessage } from "../../types";
import { FromRenderMessage, ToRenderMessage } from "../types";

const DAMPEN_FACTOR = 2;
const PLAYER_SPEED = 3;

export class PlayerPlugin {
  #camera: PerspectiveCamera;
  #postMessage: PostMessage<FromRenderMessage>;

  #playerInputVector = new Vector2();

  #playerPosition: Int32Array | null = null;
  #playerRotation: Int32Array | null = null;
  #playerVelocity: Int32Array | null = null;

  #inputMomentum = new Vector2();
  #inputYChangeTime = 0;
  #inputXChangeTime = 0;

  #tempVec2 = new Vector2();
  #tempVec3 = new Vector3();
  #tempEuler = new Euler(0, 0, 0, "YXZ");

  // Set to constrain the pitch of the camera
  #minPolarAngle = 0;
  #maxPolarAngle = Math.PI;

  #pointerSpeed = 1.0;

  constructor(
    camera: PerspectiveCamera,
    postMessage: PostMessage<FromRenderMessage>
  ) {
    this.#camera = camera;
    this.#postMessage = postMessage;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_player_buffers": {
        this.#playerPosition = data.position;
        this.#playerVelocity = data.velocity;

        // Create shared array buffer
        const rotationBuffer = new SharedArrayBuffer(
          Int32Array.BYTES_PER_ELEMENT * 4
        );

        this.#playerRotation = new Int32Array(rotationBuffer);

        // Send back to main thread
        this.#postMessage({
          subject: "set_player_rotation_buffer",
          data: this.#playerRotation,
        });

        break;
      }

      case "set_player_input_vector": {
        this.setPlayerInputVector(data);
        break;
      }

      case "mouse_move": {
        this.mouseMove(data.x, data.y);
        break;
      }
    }
  }

  mouseMove(x: number, y: number) {
    this.#tempEuler.setFromQuaternion(this.#camera.quaternion);

    this.#tempEuler.y -= x * 0.002 * this.#pointerSpeed;
    this.#tempEuler.x -= y * 0.002 * this.#pointerSpeed;

    this.#tempEuler.x = Math.max(
      Math.PI / 2 - this.#maxPolarAngle,
      Math.min(Math.PI / 2 - this.#minPolarAngle, this.#tempEuler.x)
    );

    this.#camera.quaternion.setFromEuler(this.#tempEuler);
  }

  setPlayerInputVector(input: [number, number]) {
    if (Math.sign(input[0]) !== Math.sign(this.#playerInputVector.x)) {
      this.#inputXChangeTime = Date.now();
    }

    if (Math.sign(input[1]) !== Math.sign(this.#playerInputVector.y)) {
      this.#inputYChangeTime = Date.now();
    }

    this.#playerInputVector.set(input[0], input[1]);
  }

  animate() {
    const deltaX = Date.now() - this.#inputXChangeTime;
    const deltaY = Date.now() - this.#inputYChangeTime;

    // Dampen input
    this.#inputMomentum.x = MathUtils.damp(
      this.#inputMomentum.x,
      this.#playerInputVector.x,
      DAMPEN_FACTOR,
      deltaX / 1000
    );

    this.#inputMomentum.y = MathUtils.damp(
      this.#inputMomentum.y,
      this.#playerInputVector.y,
      DAMPEN_FACTOR,
      deltaY / 1000
    );

    if (Math.abs(this.#inputMomentum.x) < 0.001) this.#inputMomentum.x = 0;
    if (Math.abs(this.#inputMomentum.y) < 0.001) this.#inputMomentum.y = 0;

    // Rotate input vector by camera direction
    const direction = this.#camera.getWorldDirection(this.#tempVec3);
    const angle = Math.atan2(direction.x, direction.z);
    const velocity = this.#tempVec2
      .set(this.#inputMomentum.x, this.#inputMomentum.y)
      .rotateAround(new Vector2(0, 0), -angle)
      .multiplyScalar(PLAYER_SPEED);

    // Send velocity
    if (this.#playerVelocity) {
      Atomics.store(this.#playerVelocity, 0, velocity.x * 1000);
      Atomics.store(this.#playerVelocity, 1, velocity.y * 1000);
    }

    // Send player rotation
    if (this.#playerRotation) {
      const rotation = this.#camera.quaternion;
      Atomics.store(this.#playerRotation, 0, rotation.x * 100000);
      Atomics.store(this.#playerRotation, 1, rotation.y * 100000);
      Atomics.store(this.#playerRotation, 2, rotation.z * 100000);
      Atomics.store(this.#playerRotation, 3, rotation.w * 100000);
    }

    // Apply player position
    if (this.#playerPosition) {
      this.#camera.position.set(
        Atomics.load(this.#playerPosition, 0) / 1000,
        Atomics.load(this.#playerPosition, 1) / 1000,
        Atomics.load(this.#playerPosition, 2) / 1000
      );
    }
  }
}
