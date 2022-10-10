import { VRM, VRMLoaderPlugin } from "@pixiv/three-vrm";
import {
  BoxGeometry,
  Group,
  Mesh,
  MeshStandardMaterial,
  Quaternion,
  Vector3,
} from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { PLAYER_HEIGHT, PLAYER_RADIUS } from "../../../constants";
import { disposeObject } from "../../utils/disposeObject";

const LERP_FACTOR = 0.000001;

export class OtherPlayer {
  readonly playerId: string;
  readonly group = new Group();

  #targetPosition = new Vector3();
  #targetRotation = new Quaternion();

  #loader = new GLTFLoader();

  constructor(playerId: string, avatarPath?: string) {
    this.playerId = playerId;

    if (!avatarPath) {
      const geometry = new BoxGeometry(
        PLAYER_RADIUS * 2,
        PLAYER_HEIGHT,
        PLAYER_RADIUS * 2
      );
      const material = new MeshStandardMaterial({ color: 0xff3333 });
      const mesh = new Mesh(geometry, material);
      mesh.position.y = PLAYER_HEIGHT / 2;
      this.group.add(mesh);
      return;
    }

    this.#loader.register((parser) => new VRMLoaderPlugin(parser));

    // Load VRM model
    this.#loader.load(
      avatarPath,
      (gltf) => {
        const vrm = gltf.userData.vrm as VRM;
        this.group.add(vrm.scene);

        vrm.scene.traverse((object) => {
          if (object instanceof Mesh) {
            object.castShadow = true;
          }
        });
      },

      (progress) => {
        const percent = (progress.loaded / progress.total) * 100;
        console.info(
          `Loading ${playerId}'s avatar: %c${percent.toFixed(2)}`,
          "color: #52DAFF",
          "%"
        );
      },

      (error) => console.error(error)
    );
  }

  setLocation(
    location: [number, number, number, number, number, number, number]
  ) {
    this.#targetPosition.set(location[0], location[1], location[2]);
    this.#targetRotation.set(
      location[3],
      location[4],
      location[5],
      location[6]
    );
  }

  animate(delta: number) {
    const K = 1 - Math.pow(LERP_FACTOR, delta);

    this.group.position.lerp(this.#targetPosition, K);
    this.group.quaternion.slerp(this.#targetRotation, K);
  }

  destroy() {
    disposeObject(this.group);
  }
}
