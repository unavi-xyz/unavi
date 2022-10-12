import { VRM, VRMLoaderPlugin } from "@pixiv/three-vrm";
import {
  AnimationAction,
  AnimationClip,
  AnimationMixer,
  BoxGeometry,
  Group,
  LoopOnce,
  Mesh,
  MeshStandardMaterial,
  Quaternion,
  Vector3,
} from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { PLAYER_HEIGHT, PLAYER_RADIUS } from "../../../constants";
import { disposeObject } from "../../utils/disposeObject";
import { loadMixamoAnimation } from "./loadMixamoAnimation";
import { AnimationName } from "./types";

const LERP_FACTOR = 0.000001;

export class OtherPlayer {
  readonly playerId: string;
  readonly group = new Group();

  #vrm: VRM | null = null;

  #mixer: AnimationMixer | null = null;
  #actions = new Map<AnimationName, AnimationAction>();

  #velocity = new Vector3();
  #targetPosition = new Vector3();
  #targetRotation = new Quaternion();

  #loader = new GLTFLoader();

  constructor(
    playerId: string,
    avatarPath?: string,
    avatarAnimationsPath?: string
  ) {
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
    try {
      this.#loadModel(avatarPath, avatarAnimationsPath);
    } catch (error) {
      console.error(error);
      console.error(`🚨 Failed to load ${this.playerId}'s avatar`);
    }
  }

  async #loadModel(avatarPath: string, avatarAnimationsPath?: string) {
    const gltf = await this.#loader.loadAsync(avatarPath);
    const vrm = gltf.userData.vrm as VRM;
    this.#vrm = vrm;

    // Add model to the scene
    this.group.add(vrm.scene);

    // Process vrm scene
    vrm.scene.traverse((object) => {
      object.frustumCulled = false;

      if (object instanceof Mesh) {
        object.castShadow = true;
      }
    });

    if (avatarAnimationsPath) {
      // Create mixer
      this.#mixer = new AnimationMixer(vrm.scene);

      // Load animations
      const animations = new Map<AnimationName, AnimationClip>();

      const clipPromises = Object.values(AnimationName).map(async (name) => {
        if (!this.#vrm) throw new Error("VRM not loaded");
        if (!this.#mixer) throw new Error("Mixer not created");

        const path = `${avatarAnimationsPath}${name}.fbx`;

        try {
          const clips = await loadMixamoAnimation(path, this.#vrm);
          const clip = clips[0];
          if (!clip) throw new Error(`No clip found for ${name}`);

          animations.set(name, clip);
        } catch (error) {
          console.error(`🚨 Failed to load ${name} animation`);
          console.error(error);
        }
      });

      await Promise.all(clipPromises);

      // Create actions
      animations.forEach((clip, name) => {
        if (!this.#mixer) throw new Error("Mixer not created");
        const action = this.#mixer.clipAction(clip);
        action.setEffectiveWeight(0);
        this.#actions.set(name, action);
      });

      this.#actions.get(AnimationName.Idle)?.play();
      this.#actions.get(AnimationName.Walk)?.play();

      this.#actions
        .get(AnimationName.Jump)
        ?.setLoop(LoopOnce, 1)
        .setEffectiveTimeScale(1.5);
    }

    console.info(`💃 Loaded ${this.playerId}'s avatar`);
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

    // Only rotate on Y axis
    this.group.quaternion.x = 0;
    this.group.quaternion.z = 0;
    this.group.quaternion.normalize();

    // Calculate velocity
    const velocity = this.#velocity
      .copy(this.#targetPosition)
      .sub(this.group.position)
      .divideScalar(delta);

    // Set animation weights
    const jumpWeight = velocity.y > 0.5 ? 1 : 0;
    const isJumping = this.#actions.get(AnimationName.Jump)?.isRunning();

    const walkWeight = isJumping ? 0 : Math.min(velocity.length() / 0.5, 1);
    const idleWeight = 1 - walkWeight - jumpWeight;

    if (jumpWeight && !isJumping) {
      // this.#actions.get(AnimationName.Jump)?.reset().play();
    } else if (!jumpWeight && isJumping) {
      // this.#actions.get(AnimationName.Jump)?.stop();
    }

    this.#actions.get(AnimationName.Walk)?.setEffectiveWeight(walkWeight);
    this.#actions.get(AnimationName.Jump)?.setEffectiveWeight(jumpWeight);
    this.#actions.get(AnimationName.Idle)?.setEffectiveWeight(idleWeight);

    // Update animations
    if (this.#mixer) this.#mixer.update(delta);

    // Update VRM
    if (this.#vrm) this.#vrm.update(delta);
  }

  destroy() {
    disposeObject(this.group);
  }
}
