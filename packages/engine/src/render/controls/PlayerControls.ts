import { VRMFirstPerson } from "@pixiv/three-vrm";
import { Euler, Group, Object3D, PerspectiveCamera, Quaternion, Raycaster, Vector3 } from "three";

import {
  INPUT_ARRAY_ROUNDING,
  PLAYER_HEIGHT,
  POSITION_ARRAY_ROUNDING,
  ROTATION_ARRAY_ROUNDING,
} from "../../constants";
import { ToRenderMessage } from "../messages";
import { Avatar } from "../players/Avatar";

const MIN_FIRST_PERSON_ANGLE = -Math.PI / 2.8;
const MAX_FIRST_PERSON_ANGLE = Math.PI / 2.8;
const MIN_THIRD_PERSON_ANGLE = -Math.PI / 2;
const MAX_THIRD_PERSON_ANGLE = Math.PI / 2;
const MAX_ORBIT_DISTANCE = 15;

/**
 * Represents the player in the render thread.
 * Reads input from the main thread, and player's position from the physics thread.
 * Updates the player's rotation, camera position, and camera rotation.
 */
export class PlayerControls {
  #camera: PerspectiveCamera;
  #root: Object3D;

  group = new Group();
  body = new Group();
  #avatar: Avatar | null = null;

  inputPosition: Int16Array | null = null;
  inputRotation: Int16Array | null = null;
  userPosition: Int32Array | null = null;
  userRotation: Int16Array | null = null;
  cameraPosition: Int32Array | null = null;
  cameraYaw: Int16Array | null = null;

  #targetCameraRotation = new Quaternion();
  #targetOrbitDistance = 0;
  #raycaster = new Raycaster();
  #euler = new Euler(0, 0, 0, "YXZ");
  #vec3 = new Vector3();
  #vec3b = new Vector3();
  #quat = new Quaternion();

  #mode: "first-person" | "third-person" = "first-person";
  #animationsPath: string | null = null;
  #defaultAvatar: string | null = null;
  #grounded = false;
  #sprinting = false;

  constructor(camera: PerspectiveCamera, root: Object3D) {
    this.#camera = camera;
    this.#root = root;

    this.group.add(this.body);
  }

  get mode() {
    return this.#mode;
  }

  set mode(mode: "first-person" | "third-person") {
    this.#mode = mode;
    if (this.avatar) this.avatar.mode = mode;

    if (mode === "first-person") {
      this.#camera.layers.enable(VRMFirstPerson.DEFAULT_FIRSTPERSON_ONLY_LAYER);
      this.#camera.layers.disable(VRMFirstPerson.DEFAULT_THIRDPERSON_ONLY_LAYER);

      // Limit camera rotation
      this.#euler.setFromQuaternion(this.#targetCameraRotation);
      this.#euler.x = Math.max(
        MIN_FIRST_PERSON_ANGLE,
        Math.min(MAX_FIRST_PERSON_ANGLE, this.#euler.x)
      );
      this.#targetCameraRotation.setFromEuler(this.#euler);
    } else {
      this.#camera.layers.disable(VRMFirstPerson.DEFAULT_FIRSTPERSON_ONLY_LAYER);
      this.#camera.layers.enable(VRMFirstPerson.DEFAULT_THIRDPERSON_ONLY_LAYER);
    }
  }

  get avatar() {
    return this.#avatar;
  }

  set avatar(avatar: Avatar | null) {
    if (avatar?.uri === this.#avatar?.uri) return;

    if (avatar && this.#avatar) {
      avatar.velocity.copy(this.#avatar.velocity);
      avatar.group.position.copy(this.#avatar.group.position);
      avatar.group.rotation.copy(this.#avatar.group.rotation);
      avatar.targetPosition.copy(this.#avatar.targetPosition);
      avatar.targetRotation.copy(this.#avatar.targetRotation);
    }

    if (avatar) {
      avatar.mode = this.mode;
      avatar.animationsPath = this.#animationsPath;
      avatar.grounded = this.#grounded;
      avatar.sprinting = this.#sprinting;
      avatar.inputPosition = this.inputPosition;
      this.body.add(avatar.group);
    }

    if (this.#avatar) this.#avatar.destroy();
    this.#avatar = avatar;
  }

  onmessage({ subject, data }: ToRenderMessage) {
    switch (subject) {
      case "set_user_arrays": {
        this.inputPosition = data.inputPosition;
        this.inputRotation = data.inputRotation;
        this.userPosition = data.userPosition;
        this.userRotation = data.userRotation;
        this.cameraPosition = data.cameraPosition;
        this.cameraYaw = data.cameraYaw;

        if (this.avatar) {
          this.avatar.inputPosition = this.inputPosition;
        }
        break;
      }

      case "mousemove": {
        if (this.inputRotation) {
          const currentX = Atomics.load(this.inputRotation, 0) / INPUT_ARRAY_ROUNDING;
          const currentY = Atomics.load(this.inputRotation, 1) / INPUT_ARRAY_ROUNDING;

          const x = currentX - data.x / 1000;
          const y = currentY - data.y / 1000;

          // Keep within -2 to 2, so we don't overflow the array
          // A rotation of 2 in either direction is a full rotation
          // So we can change it to 0 and it will be the same
          const x2 = x < -2 ? x + 2 : x > 2 ? x - 2 : x;

          Atomics.store(this.inputRotation, 0, x2 * INPUT_ARRAY_ROUNDING);
          Atomics.store(this.inputRotation, 1, y * INPUT_ARRAY_ROUNDING);
        }
        break;
      }

      case "wheel": {
        this.#targetOrbitDistance += data.deltaY / 100;
        this.#targetOrbitDistance = Math.max(
          0,
          Math.min(MAX_ORBIT_DISTANCE, this.#targetOrbitDistance)
        );

        // Switch to first person mode when zoom in
        if (this.mode === "third-person" && this.#targetOrbitDistance < 0.3) {
          this.mode = "first-person";
          this.#targetOrbitDistance = 0;
        }

        // Switch to third person mode when zoom out
        if (this.mode === "first-person" && this.#targetOrbitDistance > 0) {
          this.mode = "third-person";
          this.#targetOrbitDistance = 0.4;

          // Reset head rotation
          if (this.avatar) {
            const direction = this.#camera.getWorldDirection(this.#vec3);
            const angle = Math.atan2(direction.x, direction.z) + Math.PI;
            this.avatar.targetRotation.setFromAxisAngle(this.#vec3.set(0, 1, 0), angle);
          }
        }
        break;
      }

      case "set_default_avatar": {
        this.#defaultAvatar = data;
        if (!this.avatar) this.avatar = new Avatar(data, this.#camera);
        break;
      }

      case "set_user_avatar": {
        if (data) this.avatar = new Avatar(data, this.#camera);
        else if (this.#defaultAvatar) this.avatar = new Avatar(this.#defaultAvatar, this.#camera);
        else this.avatar = null;
        break;
      }

      case "set_animations_path": {
        this.#animationsPath = data;
        if (this.avatar) this.avatar.animationsPath = data;
        break;
      }

      case "set_grounded": {
        this.#grounded = data;
        if (this.avatar) this.avatar.grounded = data;
        break;
      }

      case "set_sprinting": {
        this.#sprinting = data;
        if (this.avatar) this.avatar.sprinting = data;
        break;
      }
    }
  }

  applyInputRotation() {
    if (!this.inputRotation) return;

    const x = Atomics.load(this.inputRotation, 0) / INPUT_ARRAY_ROUNDING;
    const y = Atomics.load(this.inputRotation, 1) / INPUT_ARRAY_ROUNDING;

    this.#euler.setFromQuaternion(this.#targetCameraRotation);

    this.#euler.y = x * Math.PI;
    this.#euler.x = y * Math.PI;

    if (this.mode === "first-person") {
      const minX = Math.min(MAX_FIRST_PERSON_ANGLE, this.#euler.x);
      const maxX = Math.max(MIN_FIRST_PERSON_ANGLE, minX);

      if (minX !== this.#euler.x) {
        this.#euler.x = minX;
        Atomics.store(this.inputRotation, 1, (minX / Math.PI) * INPUT_ARRAY_ROUNDING);
      } else if (maxX !== this.#euler.x) {
        this.#euler.x = maxX;
        Atomics.store(this.inputRotation, 1, (maxX / Math.PI) * INPUT_ARRAY_ROUNDING);
      }
    } else {
      const minX = Math.min(MAX_THIRD_PERSON_ANGLE, this.#euler.x);
      const maxX = Math.max(MIN_THIRD_PERSON_ANGLE, minX);

      if (minX !== this.#euler.x) {
        this.#euler.x = minX;
        Atomics.store(this.inputRotation, 1, (minX / Math.PI) * INPUT_ARRAY_ROUNDING);
      } else if (maxX !== this.#euler.x) {
        this.#euler.x = maxX;
        Atomics.store(this.inputRotation, 1, (maxX / Math.PI) * INPUT_ARRAY_ROUNDING);
      }
    }

    this.#targetCameraRotation.setFromEuler(this.#euler);
  }

  update(delta: number) {
    if (
      !this.inputRotation ||
      !this.userPosition ||
      !this.userRotation ||
      !this.cameraYaw ||
      !this.cameraPosition ||
      !this.avatar
    )
      return;

    // Apply camera movement
    this.applyInputRotation();

    // Set target position
    const posX = Atomics.load(this.userPosition, 0) / POSITION_ARRAY_ROUNDING;
    const posY = Atomics.load(this.userPosition, 1) / POSITION_ARRAY_ROUNDING;
    const posZ = Atomics.load(this.userPosition, 2) / POSITION_ARRAY_ROUNDING;
    this.body.position.set(posX, posY, posZ);

    this.avatar.update(delta);

    if (this.mode === "first-person") this.updateFirstPerson(delta);
    else this.updateThirdPerson(delta);

    // Store camera rotation
    const direction = this.#camera.getWorldDirection(this.#vec3);
    const yaw = Math.atan2(direction.x, direction.z);
    Atomics.store(this.cameraYaw, 0, yaw * ROTATION_ARRAY_ROUNDING);

    // Store camera position
    this.#camera.getWorldPosition(this.#vec3);
    Atomics.store(this.cameraPosition, 0, this.#vec3.x * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.cameraPosition, 1, this.#vec3.y * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.cameraPosition, 2, this.#vec3.z * POSITION_ARRAY_ROUNDING);
  }

  updateFirstPerson(delta: number) {
    if (!this.userRotation || !this.avatar) return;

    // Rotate camera
    const K = 1 - Math.pow(10e-18, delta);
    this.#camera.quaternion.slerp(this.#targetCameraRotation, K);

    // Set avatar rotation
    this.#camera.getWorldQuaternion(this.avatar.targetRotation);
    this.avatar.group.quaternion.copy(this.avatar.targetRotation);
    this.avatar.group.quaternion.x = 0;
    this.avatar.group.quaternion.z = 0;
    this.avatar.group.quaternion.normalize();

    // Move camera
    const head = this.avatar.vrm?.humanoid.getNormalizedBoneNode("head");
    const leftEye = this.avatar.vrm?.humanoid.getNormalizedBoneNode("leftEye");
    const rightEye = this.avatar.vrm?.humanoid.getNormalizedBoneNode("rightEye");

    if (leftEye && rightEye) {
      // Move camera to eye position
      this.#camera.position
        .addVectors(leftEye.getWorldPosition(this.#vec3), rightEye.getWorldPosition(this.#vec3b))
        .multiplyScalar(0.5);
    } else if (head) {
      // Move camera to head position
      head.getWorldPosition(this.#camera.position);
    } else {
      // Move camera to body position + height offset
      this.body.getWorldPosition(this.#camera.position);
      this.#camera.position.y += PLAYER_HEIGHT;
    }

    // Move camera forward a bit
    const direction = this.#camera.getWorldDirection(this.#vec3);
    this.#camera.position.add(this.#vec3.copy(direction).multiplyScalar(0.15));

    // Store user rotation
    this.#camera.getWorldQuaternion(this.#quat);
    Atomics.store(this.userRotation, 0, this.#quat.x * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.userRotation, 1, this.#quat.y * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.userRotation, 2, this.#quat.z * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.userRotation, 3, this.#quat.w * ROTATION_ARRAY_ROUNDING);
  }

  updateThirdPerson(delta: number) {
    if (!this.userRotation || !this.inputPosition || !this.avatar) return;

    // Rotate camera
    const K = 1 - Math.pow(10e-14, delta);
    this.#camera.quaternion.slerp(this.#targetCameraRotation, K);

    // Move camera
    const head = this.avatar.vrm?.humanoid.getNormalizedBoneNode("head");
    if (head) {
      // Orbit around head
      head.getWorldPosition(this.#camera.position);
    } else {
      // Orbit around body + height offset
      this.body.getWorldPosition(this.#camera.position).add(this.#vec3.set(0, PLAYER_HEIGHT, 0));
    }

    // Get orbit distance
    this.#raycaster.set(
      this.#camera.position,
      this.#camera.getWorldDirection(this.#vec3).multiplyScalar(-1)
    );
    const intersection = this.#raycaster
      .intersectObject(this.#root)
      .find(({ object }) => object.visible);
    const maxOrbitDistance = intersection ? intersection.distance - 0.1 : MAX_ORBIT_DISTANCE;
    const orbitDistance = Math.max(0, Math.min(maxOrbitDistance, this.#targetOrbitDistance));

    // Move camera to orbit distance
    this.#camera.position.add(
      this.#vec3.set(0, 0, orbitDistance).applyQuaternion(this.#camera.quaternion)
    );

    // Rotate body
    if (this.avatar.velocity.length() > 0.01) {
      this.avatar.targetRotation.setFromAxisAngle(
        this.#vec3.set(0, 1, 0),
        Math.atan2(this.avatar.velocity.x, this.avatar.velocity.z)
      );
    }

    // Store user rotation
    this.avatar.group.getWorldQuaternion(this.#quat);
    Atomics.store(this.userRotation, 0, this.#quat.x * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.userRotation, 1, this.#quat.y * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.userRotation, 2, this.#quat.z * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.userRotation, 3, this.#quat.w * ROTATION_ARRAY_ROUNDING);
  }
}
