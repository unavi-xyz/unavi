import { VRM, VRMLoaderPlugin } from "@pixiv/three-vrm";
import {
  AnimationAction,
  AnimationClip,
  AnimationMixer,
  BoxGeometry,
  Group,
  LoopPingPong,
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

  isFalling = false;

  #vrm: VRM | null = null;

  #defaultAvatarPath?: string;
  #avatarAnimationsPath?: string;
  #mixer: AnimationMixer | null = null;
  #actions = new Map<AnimationName, AnimationAction>();

  #fallWeight = 0;
  #leftWeight = 0;
  #rightWeight = 0;
  #forwardWeight = 0;

  #velocity = new Vector3();
  #tempQuat = new Quaternion();
  #targetPosition = new Vector3();
  #targetRotation = new Quaternion();

  #loader = new GLTFLoader();

  constructor(
    playerId: string,
    avatarPath?: string,
    avatarAnimationsPath?: string
  ) {
    this.playerId = playerId;
    this.#defaultAvatarPath = avatarPath;
    this.#avatarAnimationsPath = avatarAnimationsPath;

    this.#loader.register((parser) => new VRMLoaderPlugin(parser));

    // Load VRM model
    try {
      this.#loadModel(avatarPath, avatarAnimationsPath);
    } catch (error) {
      console.error(error);
      console.error(`🚨 Failed to load ${this.playerId}'s avatar`);
    }
  }

  async #loadModel(avatarPath?: string, avatarAnimationsPath?: string) {
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

    const gltf = await this.#loader.loadAsync(avatarPath);
    const vrm = gltf.userData.vrm as VRM;

    // Remove previous VRM model
    if (this.#vrm) {
      this.#vrm.scene.removeFromParent();
      disposeObject(this.#vrm.scene);
      this.#vrm = null;
    }

    // Remove previous mixer
    if (this.#mixer) {
      this.#mixer.stopAllAction();
      this.#actions.clear();
      this.#mixer = null;
    }

    // Set VRM model
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
      this.#actions.get(AnimationName.LeftWalk)?.play();
      this.#actions.get(AnimationName.RightWalk)?.play();

      this.#actions
        .get(AnimationName.Falling)
        ?.play()
        .setLoop(LoopPingPong, Infinity);
    }

    console.info(`💃 Loaded ${this.playerId}'s avatar`);
  }

  setAvatar(avatarPath: string | null) {
    this.#loadModel(
      avatarPath ?? this.#defaultAvatarPath,
      this.#avatarAnimationsPath
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

    // Only rotate on Y axis
    this.group.quaternion.x = 0;
    this.group.quaternion.z = 0;
    this.group.quaternion.normalize();

    // Calculate velocity relative to player rotation
    const velocity = this.#velocity
      .copy(this.#targetPosition)
      .sub(this.group.position)
      .divideScalar(delta)
      .applyQuaternion(this.#tempQuat.copy(this.group.quaternion).invert());

    // Falling
    this.#fallWeight = clamp(
      this.isFalling
        ? this.#fallWeight + delta * 4
        : this.#fallWeight - delta * 4
    );

    this.#actions
      .get(AnimationName.Falling)
      ?.setEffectiveWeight(this.#fallWeight);

    // Walking
    const leftVelocity = velocity.x < 0 ? -velocity.x : 0;
    const rightVelocity = velocity.x > 0 ? velocity.x : 0;
    const forwardVelocity = Math.abs(velocity.z);
    const isBackwards = velocity.z > 0;

    this.#leftWeight = clamp(
      leftVelocity > 1 && !this.isFalling
        ? this.#leftWeight + delta * 4
        : this.#leftWeight - delta * 4
    );

    this.#rightWeight = clamp(
      rightVelocity > 1 && !this.isFalling
        ? this.#rightWeight + delta * 4
        : this.#rightWeight - delta * 4
    );

    this.#forwardWeight = clamp(
      forwardVelocity > 1 && !this.isFalling
        ? this.#forwardWeight + delta * 4
        : this.#forwardWeight - delta * 4
    );

    this.#actions
      .get(AnimationName.LeftWalk)
      ?.setEffectiveWeight(this.#leftWeight);

    this.#actions
      .get(AnimationName.RightWalk)
      ?.setEffectiveWeight(this.#rightWeight);

    this.#actions
      .get(AnimationName.Walk)
      ?.setEffectiveWeight(this.#forwardWeight)
      .setEffectiveTimeScale(isBackwards ? -1 : 1);

    // Idle
    const idleWeight =
      1 -
      this.#leftWeight -
      this.#rightWeight -
      this.#forwardWeight -
      this.#fallWeight;

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

function clamp(value: number) {
  return Math.max(Math.min(value, 1), 0);
}
