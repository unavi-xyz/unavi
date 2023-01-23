import { VRMFirstPerson } from "@pixiv/three-vrm";
import { Euler, Group, PerspectiveCamera, Quaternion, Vector3 } from "three";
import { CSM } from "three/examples/jsm/csm/CSM";

import { PLAYER_HEIGHT, POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "../../constants";
import { ToRenderMessage } from "../messages";
import { Avatar } from "../players/Avatar";

export class PlayerControls {
  #camera: PerspectiveCamera;

  group = new Group();
  body = new Group();
  avatar: Avatar | null = null;

  positionArray: Int32Array | null = null;
  rotationArray: Int32Array | null = null;

  #targetCameraRotation = new Quaternion();
  #euler = new Euler(0, 0, 0, "YXZ");
  #vec3 = new Vector3();
  #delta = 0;
  #mode: "first-person" | "third-person" = "first-person";
  #csm: CSM | null = null;

  constructor(camera: PerspectiveCamera) {
    this.#camera = camera;
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
    } else {
      this.#camera.layers.disable(VRMFirstPerson.DEFAULT_FIRSTPERSON_ONLY_LAYER);
      this.#camera.layers.enable(VRMFirstPerson.DEFAULT_THIRDPERSON_ONLY_LAYER);
    }
  }

  get csm() {
    return this.#csm;
  }

  set csm(csm: CSM | null) {
    this.#csm = csm;
    if (this.avatar) this.avatar.csm = csm;
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

      case "set_player_avatar": {
        if (this.avatar) {
          this.avatar.dispose();
          this.avatar = null;
        }

        if (data.uri) {
          this.avatar = new Avatar(data.uri);
          this.avatar.mode = this.mode;
          this.avatar.csm = this.csm;
          this.body.add(this.avatar.group);
        }
        break;
      }
    }
  }

  update(delta: number) {
    this.#delta = delta;
    if (this.avatar) this.avatar.update(delta);
    if (!this.positionArray || !this.rotationArray) return;

    // Move body to player position
    const x = Atomics.load(this.positionArray, 0) / POSITION_ARRAY_ROUNDING;
    const y = Atomics.load(this.positionArray, 1) / POSITION_ARRAY_ROUNDING;
    const z = Atomics.load(this.positionArray, 2) / POSITION_ARRAY_ROUNDING;
    this.body.position.set(x, y, z);

    if (this.mode === "first-person") this.updateFirstPerson(delta);
    else this.updateThirdPerson(delta);
  }

  updateFirstPerson(delta: number) {
    if (!this.positionArray || !this.rotationArray) return;

    // Move camera
    const head = this.avatar?.vrm?.humanoid.getNormalizedBoneNode("head");
    if (head) {
      // Move camera to head position
      head.getWorldPosition(this.#camera.position);
    } else {
      // Move camera to body position + height offset if no head
      this.body.getWorldPosition(this.#camera.position);
      this.#camera.position.y += PLAYER_HEIGHT;
    }

    // Rotate camera
    const K = 1 - Math.pow(10e-24, delta);
    this.#camera.quaternion.slerp(this.#targetCameraRotation, K);

    // Store camera rotation
    const direction = this.#camera.getWorldDirection(this.#vec3);
    const angle = Math.atan2(direction.x, direction.z);
    Atomics.store(this.rotationArray, 0, angle * ROTATION_ARRAY_ROUNDING);

    // Rotate body around Y axis
    this.body.rotation.y = angle;
  }

  updateThirdPerson(delta: number) {
    if (!this.positionArray || !this.rotationArray) return;
  }
}
