import {
  Euler,
  Group,
  MathUtils,
  PerspectiveCamera,
  Quaternion,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";

import { PLAYER_HEIGHT } from "../../constants";
import { PostMessage } from "../../types";
import { FromRenderMessage, ToRenderMessage } from "../types";
import { PlayerAvatar } from "./OtherPlayers/PlayerAvatar";
import { RenderPlugin } from "./types";

const LERP_FACTOR = 10e-24;
const DAMPEN_FACTOR = 2;
const PLAYER_SPEED = 3;
const THIRD_PERSON_OFFSET = new Vector3(0, PLAYER_HEIGHT * 0.8, 0);
const UP = new Vector3(0, 1, 0);
const VEC2 = new Vector2();

export class PlayerPlugin implements RenderPlugin {
  readonly group = new Group();

  #camera: PerspectiveCamera;
  #postMessage: PostMessage<FromRenderMessage>;
  #avatarPath?: string;
  #avatarAnimationsPath?: string;
  #renderer: WebGLRenderer;

  #avatar: PlayerAvatar | null = null;

  #playerInputVector = new Vector2();
  #playerVelocity: Int32Array | null = null;
  #playerPosition: Int32Array | null = null;
  #playerRotation: Int16Array | null = null;

  #inputMomentum = new Vector2();
  #inputYChangeTime = 0;
  #inputXChangeTime = 0;

  #tempVec2 = new Vector2();
  #tempVec3 = new Vector3();
  #cameraRotation = new Euler(0, 0, 0, "YXZ");
  #targetCameraRotation = new Quaternion();
  #targetCameraPosition = new Vector3();
  #walkingDirection = new Vector2();
  #hasWalked = false;
  #delta = 0;

  constructor(
    camera: PerspectiveCamera,
    postMessage: PostMessage<FromRenderMessage>,
    renderer: WebGLRenderer,
    avatarPath?: string,
    avatarAnimationsPath?: string
  ) {
    this.#camera = camera;
    this.#postMessage = postMessage;
    this.#avatarPath = avatarPath;
    this.#avatarAnimationsPath = avatarAnimationsPath;
    this.#renderer = renderer;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "set_player_buffers": {
        this.#playerPosition = data.position;
        this.#playerVelocity = data.velocity;

        // Create shared array buffer
        const rotationBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4);

        this.#playerRotation = new Int16Array(rotationBuffer);

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

      case "set_avatar": {
        if (this.#avatar) {
          this.#avatar.group.removeFromParent();
          this.#avatar.destroy();
        }

        this.#avatar = new PlayerAvatar(
          -1,
          data,
          this.#postMessage,
          this.#camera,
          this.#renderer,
          this.#avatarPath,
          this.#avatarAnimationsPath,
          this.#camera
        );

        this.group.add(this.#avatar.group);
        break;
      }

      case "set_player_falling_state": {
        if (data.playerId === -1 && this.#avatar) {
          this.#avatar.isFalling = data.isFalling;
        }
        break;
      }

      case "wheel": {
        this.wheel(data.deltaY);
        break;
      }
    }
  }

  wheel(delta: number) {
    // Scroll out into third person mode
    this.#targetCameraPosition.z += delta / 100;
    this.#targetCameraPosition.z = Math.max(0, Math.min(20, this.#targetCameraPosition.z));
  }

  mouseMove(x: number, y: number) {
    this.#cameraRotation.setFromQuaternion(this.#targetCameraRotation);

    this.#cameraRotation.y -= (x * this.#delta) / 5;
    this.#cameraRotation.x -= (y * this.#delta) / 5;

    const minPolarAngle = 0;
    const maxPolarAngle = Math.PI;

    this.#cameraRotation.x = Math.max(
      Math.PI / 2 - maxPolarAngle,
      Math.min(Math.PI / 2 - minPolarAngle, this.#cameraRotation.x)
    );

    this.#targetCameraRotation.setFromEuler(this.#cameraRotation);
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

  update(delta: number) {
    this.#delta = delta;
    const K = 1 - Math.pow(LERP_FACTOR, delta);

    // Dampen input
    const deltaX = Date.now() - this.#inputXChangeTime;
    const deltaY = Date.now() - this.#inputYChangeTime;

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
      .rotateAround(VEC2, -angle)
      .multiplyScalar(PLAYER_SPEED);

    // Send velocity
    if (this.#playerVelocity) {
      Atomics.store(this.#playerVelocity, 0, velocity.x * 1000);
      Atomics.store(this.#playerVelocity, 1, velocity.y * 1000);
    }

    // Apply player position
    if (this.#playerPosition && this.#avatar) {
      this.#avatar.group.position.set(
        Atomics.load(this.#playerPosition, 0) / 1000,
        Atomics.load(this.#playerPosition, 1) / 1000,
        Atomics.load(this.#playerPosition, 2) / 1000
      );
    }

    // Move camera to target position
    this.#camera.position.copy(this.#targetCameraPosition);

    // Set first person mode
    const isFirstPerson = this.#targetCameraPosition.z === 0;
    if (isFirstPerson !== this.#avatar?.isFirstPerson) this.#avatar?.setFirstPerson(isFirstPerson);

    // Rotate camera
    if (isFirstPerson) this.#camera.quaternion.slerp(this.#targetCameraRotation, K);
    else this.#camera.quaternion.copy(this.#targetCameraRotation);

    // Rotate avatar
    if (isFirstPerson) {
      // If first person, copy camera rotation to avatar
      this.#avatar?.group.quaternion.copy(this.#camera.quaternion);
      this.#avatar?.setRotation(
        this.#camera.quaternion.x,
        this.#camera.quaternion.y,
        this.#camera.quaternion.z,
        this.#camera.quaternion.w
      );

      this.#hasWalked = false;
    } else {
      // If third person, rotate avatar by walking direction
      if (velocity.x !== 0 || velocity.y !== 0) this.#hasWalked = true;

      if (this.#hasWalked) {
        this.#walkingDirection.add(velocity);
        this.#walkingDirection.clampLength(0, 1);
        const angle = Math.atan2(-this.#walkingDirection.x, -this.#walkingDirection.y);
        this.#avatar?.group.quaternion.setFromAxisAngle(UP, angle);
      }

      // Reset head rotation
      this.#avatar?.setRotation(0, 0, 0, 1);
    }

    // Update avatar
    if (this.#avatar) this.#avatar.update(delta);

    if (this.#avatar?.vrm) {
      const humanBones = this.#avatar.vrm.humanoid.humanBones;
      const head = humanBones.head.node;

      if (isFirstPerson) {
        // If first person, copy head position to camera
        this.#tempVec3.setScalar(0);

        // If eyes are available, use eye position
        if (humanBones.leftEye && humanBones.rightEye) {
          this.#tempVec3
            .copy(humanBones.leftEye.node.position)
            .add(humanBones.rightEye.node.position)
            .divideScalar(2);
        }

        head.position.add(this.#tempVec3);
        head.getWorldPosition(this.#camera.position);
        head.position.sub(this.#tempVec3);
      } else {
        // If third person, rotate camera around avatar head
        this.#camera.position
          .applyEuler(this.#cameraRotation)
          .add(this.#avatar.group.position)
          .add(THIRD_PERSON_OFFSET);
      }
    }

    // Send player rotation
    if (this.#playerRotation) {
      const rotation = isFirstPerson ? this.#camera.quaternion : this.#avatar?.group.quaternion;

      if (rotation) {
        Atomics.store(this.#playerRotation, 0, rotation.x * 1000);
        Atomics.store(this.#playerRotation, 1, rotation.y * 1000);
        Atomics.store(this.#playerRotation, 2, rotation.z * 1000);
        Atomics.store(this.#playerRotation, 3, rotation.w * 1000);
      }
    }
  }

  destroy() {
    if (this.#avatar) {
      this.#avatar.group.removeFromParent();
      this.#avatar.destroy();
    }
  }
}
