import { VRMFirstPerson } from "@pixiv/three-vrm";
import { Euler, Group, Object3D, PerspectiveCamera, Quaternion, Raycaster, Vector3 } from "three";

import { PLAYER_HEIGHT, POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "../../constants";
import { ToRenderMessage } from "../messages";
import { Avatar } from "../players/Avatar";

const MIN_FIRST_PERSON_ANGLE = -Math.PI / 2.8;
const MAX_FIRST_PERSON_ANGLE = Math.PI / 2.8;
const MAX_ORBIT_DISTANCE = 15;

export class PlayerControls {
  #camera: PerspectiveCamera;
  #root: Object3D;

  group = new Group();
  body = new Group();
  avatar: Avatar | null = null;

  positionArray: Int32Array | null = null;
  rotationArray: Int32Array | null = null;

  #targetCameraRotation = new Quaternion();
  #targetOrbitDistance = 0;
  #position = new Vector3();
  #velocity = new Vector3();
  #raycaster = new Raycaster();
  #euler = new Euler(0, 0, 0, "YXZ");
  #vec3 = new Vector3();
  #vec3b = new Vector3();
  #delta = 0;
  #mode: "first-person" | "third-person" = "first-person";

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

        if (this.mode === "first-person") {
          this.#euler.x = Math.max(
            MIN_FIRST_PERSON_ANGLE,
            Math.min(MAX_FIRST_PERSON_ANGLE, this.#euler.x)
          );
        } else {
          this.#euler.x = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, this.#euler.x));
        }

        this.#targetCameraRotation.setFromEuler(this.#euler);
        break;
      }

      case "set_player_avatar": {
        if (this.avatar) {
          this.avatar.dispose();
          this.avatar = null;
        }

        if (data.uri) {
          this.avatar = new Avatar(data.uri);
          this.avatar.mode = this.mode;
          this.body.add(this.avatar.group);
        }
        break;
      }

      case "wheel": {
        this.#targetOrbitDistance += data.deltaY * this.#delta * 0.75;
        this.#targetOrbitDistance = Math.max(
          0,
          Math.min(MAX_ORBIT_DISTANCE, this.#targetOrbitDistance)
        );

        // Switch to first person mode when zoom in
        if (this.mode === "third-person" && this.#targetOrbitDistance < 0.4) {
          this.mode = "first-person";
          this.#targetOrbitDistance = 0;
        }

        // Switch to third person mode when zoom out
        if (this.mode === "first-person" && this.#targetOrbitDistance > 0) {
          this.mode = "third-person";
          this.#targetOrbitDistance = 0.4;
        }
        break;
      }
    }
  }

  update(delta: number) {
    this.#delta = delta;
    if (!this.positionArray || !this.rotationArray) return;

    // Move body to player position
    const x = Atomics.load(this.positionArray, 0) / POSITION_ARRAY_ROUNDING;
    const y = Atomics.load(this.positionArray, 1) / POSITION_ARRAY_ROUNDING;
    const z = Atomics.load(this.positionArray, 2) / POSITION_ARRAY_ROUNDING;
    this.body.position.set(x, y, z);

    // Calculate velocity
    const K = 1 - Math.pow(10e-12, delta);
    const deltaPos = this.#vec3.set(x, y, z).sub(this.#position);
    this.#velocity.lerp(deltaPos, K);
    this.#position.set(x, y, z);

    if (this.mode === "first-person") this.updateFirstPerson(delta);
    else this.updateThirdPerson(delta);

    if (this.avatar) this.avatar.update(delta);
  }

  updateFirstPerson(delta: number) {
    if (!this.positionArray || !this.rotationArray) return;

    // Rotate camera
    const K = 1 - Math.pow(10e-18, delta);
    this.#camera.quaternion.slerp(this.#targetCameraRotation, K);

    // Set avatar rotation
    if (this.avatar) this.#camera.getWorldQuaternion(this.avatar.targetRotation);

    // Move camera
    const head = this.avatar?.vrm?.humanoid.getNormalizedBoneNode("head");
    const leftEye = this.avatar?.vrm?.humanoid.getNormalizedBoneNode("leftEye");
    const rightEye = this.avatar?.vrm?.humanoid.getNormalizedBoneNode("rightEye");

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
    this.#camera.position.add(this.#vec3.copy(direction).multiplyScalar(0.1));

    // Store camera rotation
    const angle = Math.atan2(direction.x, direction.z);
    Atomics.store(this.rotationArray, 0, angle * ROTATION_ARRAY_ROUNDING);
  }

  updateThirdPerson(delta: number) {
    if (!this.positionArray || !this.rotationArray) return;

    // Rotate camera
    const K = 1 - Math.pow(10e-14, delta);
    this.#camera.quaternion.slerp(this.#targetCameraRotation, K);

    // Move camera
    const head = this.avatar?.vrm?.humanoid.getNormalizedBoneNode("head");
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

    // Store camera rotation
    const direction = this.#camera.getWorldDirection(this.#vec3);
    const angle = Math.atan2(direction.x, direction.z);
    Atomics.store(this.rotationArray, 0, angle * ROTATION_ARRAY_ROUNDING);

    // Rotate body according to velocity
    const velocity = this.#velocity.length();
    if (this.avatar && velocity > 0.01) {
      this.avatar.targetRotation.setFromAxisAngle(
        this.#vec3.set(0, 1, 0),
        Math.atan2(this.#velocity.x, this.#velocity.z) + Math.PI
      );
    }
  }
}
