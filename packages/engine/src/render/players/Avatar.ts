import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import { AnimationAction, AnimationMixer, Group, Mesh, Quaternion, Vector3 } from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { loadMixamoAnimation } from "./loadMixamoAnimation";

const MIN_WALK_SPEED = 1;
const MAX_WALK_SPEED = 25;
const SPRINT_CUTOFF = MAX_WALK_SPEED + 5;

const ANIMATION_NAME = {
  Falling: "Falling",
  Idle: "Idle",
  LeftWalk: "LeftWalk",
  RightWalk: "RightWalk",
  Sprint: "Sprint",
  Walk: "Walk",
} as const;

export class Avatar {
  group = new Group();
  vrm: VRM | null = null;

  mixer: AnimationMixer | null = null;
  animations = new Map<keyof typeof ANIMATION_NAME, AnimationAction>();
  weights = { left: 0, right: 0, walk: 0, sprint: 0, fall: 0 };

  #quat = new Quaternion();
  #quatb = new Quaternion();
  #vec3 = new Vector3();

  #mode: "first-person" | "third-person" = "third-person";
  #animationsPath: string | null = null;
  grounded = false;

  velocity = new Vector3();
  targetPosition = new Vector3();
  targetRotation = new Quaternion();

  constructor(uri: string) {
    const loader = new GLTFLoader();
    loader.setCrossOrigin("anonymous");
    loader.register((parser) => new VRMLoaderPlugin(parser));

    loader.load(uri, (gltf) => {
      const vrm = gltf.userData.vrm as VRM;
      vrm.scene.rotateY(Math.PI);
      this.vrm = vrm;

      VRMUtils.removeUnnecessaryVertices(vrm.scene);
      VRMUtils.removeUnnecessaryJoints(vrm.scene);
      VRMUtils.rotateVRM0(vrm);

      vrm.scene.traverse((object) => {
        if (object instanceof Mesh) object.castShadow = true;
      });

      if (this.#mode === "first-person") vrm.firstPerson?.setup();

      this.group.add(vrm.scene);

      this.loadAnimations(vrm);
    });
  }

  get mode() {
    return this.#mode;
  }

  set mode(mode: "first-person" | "third-person") {
    this.#mode = mode;
    if (mode === "first-person") this.vrm?.firstPerson?.setup();
  }

  get animationsPath() {
    return this.#animationsPath;
  }

  set animationsPath(path: string | null) {
    this.#animationsPath = path;
    if (this.vrm) this.loadAnimations(this.vrm);
  }

  async loadAnimations(vrm: VRM) {
    // Clean up previous animations
    this.animations.clear();
    this.mixer?.stopAllAction();

    if (!this.animationsPath) return;

    // Load animations
    this.mixer = new AnimationMixer(vrm.scene);

    await Promise.all(
      Object.values(ANIMATION_NAME).map(async (name) => {
        if (!this.mixer || !this.animationsPath) return;

        const path = `${this.animationsPath}/${name}.fbx`;
        const clips = await loadMixamoAnimation(path, vrm);
        const clip = clips[0];
        if (!clip) return;

        const action = this.mixer.clipAction(clip).setEffectiveWeight(0);
        this.animations.set(name, action);
      })
    );
  }

  update(delta: number) {
    if (!this.vrm) return;

    this.vrm.update(delta);

    // Calculate velocity
    const K = 1 - Math.pow(10e-12, delta);
    const deltaPos = this.#vec3
      .copy(this.group.position)
      .sub(this.targetPosition)
      .divideScalar(delta);
    this.velocity.lerp(deltaPos, K);

    // Move body
    this.group.position.lerp(this.targetPosition, K);

    // Apply animations
    if (this.mixer) {
      const relativeVelocity = this.#vec3
        .copy(this.velocity)
        .applyQuaternion(this.group.getWorldQuaternion(this.#quat).invert());

      const leftVelocity = relativeVelocity.x > 0 ? Math.abs(relativeVelocity.x) : 0;
      const rightVelocity = relativeVelocity.x < 0 ? Math.abs(relativeVelocity.x) : 0;
      const forwardVelocity = Math.abs(relativeVelocity.z);
      const totalVelocity = Math.abs(relativeVelocity.x) + Math.abs(relativeVelocity.z);
      const isBackwards = relativeVelocity.z < 0;

      this.weights.walk = clamp(
        clamp(
          forwardVelocity > MIN_WALK_SPEED && this.grounded
            ? this.weights.walk + delta * 8
            : this.weights.walk - delta * 8
        ) - this.weights.sprint
      );

      this.weights.sprint = clamp(
        totalVelocity > SPRINT_CUTOFF && this.grounded
          ? this.weights.sprint + delta * 8
          : this.weights.sprint - delta * 8
      );

      this.weights.left = clamp(
        clamp(
          leftVelocity > MIN_WALK_SPEED && this.grounded
            ? this.weights.left + delta * 8
            : this.weights.left - delta * 8
        ) -
          this.weights.walk -
          this.weights.sprint
      );

      this.weights.right = clamp(
        clamp(
          rightVelocity > MIN_WALK_SPEED && this.grounded
            ? this.weights.right + delta * 8
            : this.weights.right - delta * 8
        ) -
          this.weights.walk -
          this.weights.sprint
      );

      this.weights.fall = clamp(
        this.grounded ? this.weights.fall - delta * 4 : this.weights.fall + delta * 4
      );

      const leftWalk = this.animations.get(ANIMATION_NAME.LeftWalk);
      const rightWalk = this.animations.get(ANIMATION_NAME.RightWalk);
      const walk = this.animations.get(ANIMATION_NAME.Walk);
      const sprint = this.animations.get(ANIMATION_NAME.Sprint);
      const fall = this.animations.get(ANIMATION_NAME.Falling);
      const idle = this.animations.get(ANIMATION_NAME.Idle);

      if (leftWalk) {
        if (leftWalk.isRunning() && this.weights.left === 0) leftWalk.stop();
        if (!leftWalk.isRunning() && this.weights.left > 0) leftWalk.play();
        leftWalk.setEffectiveWeight(this.weights.left);
      }

      if (rightWalk) {
        if (rightWalk.isRunning() && this.weights.right === 0) rightWalk.stop();
        if (!rightWalk.isRunning() && this.weights.right > 0) rightWalk.play();
        rightWalk.setEffectiveWeight(this.weights.right);
      }

      if (walk) {
        if (walk.isRunning() && this.weights.walk === 0 && this.weights.sprint === 0) walk.stop();
        if (!walk.isRunning() && this.weights.walk > 0) walk.play();
        walk.setEffectiveWeight(this.weights.walk);
        walk.setEffectiveTimeScale(isBackwards ? -1 : 1);
      }

      if (sprint) {
        if (sprint.isRunning() && this.weights.sprint === 0) sprint.stop();
        if (!sprint.isRunning() && this.weights.sprint > 0) sprint.play();
        sprint.setEffectiveWeight(this.weights.sprint);
        sprint.setEffectiveTimeScale(isBackwards ? -1 : 1);
      }

      if (fall) {
        if (fall.isRunning() && this.weights.fall === 0) fall.stop();
        if (!fall.isRunning() && this.weights.fall > 0) fall.play();
        fall.setEffectiveWeight(this.weights.fall);
      }

      const idleWeight = 1 - Object.values(this.weights).reduce((a, b) => a + b, 0);
      if (idle) {
        if (idle.isRunning() && idleWeight === 0) idle.stop();
        if (!idle.isRunning() && idleWeight > 0) idle.play();
        idle.setEffectiveWeight(idleWeight);
      }

      this.mixer.update(delta);
    }

    // Rotate body around Y axis
    this.group.quaternion.copy(this.targetRotation);
    this.group.quaternion.x = 0;
    this.group.quaternion.z = 0;
    this.group.quaternion.normalize();

    // Rotate head up and down
    const head = this.vrm.humanoid.getNormalizedBoneNode("head");
    if (head) {
      // Get relative rotation
      const relativeRotation = this.#quat
        .copy(this.targetRotation)
        .premultiply(this.#quatb.copy(this.group.quaternion).invert());

      // Don't rotate Y axis
      relativeRotation.y = 0;
      relativeRotation.normalize();

      if (this.vrm.meta.metaVersion === "1") {
        // If vrm 1.0, rotate axis
        const rotated = this.#quatb.copy(relativeRotation);
        rotated.x = relativeRotation.z;
        rotated.z = -relativeRotation.x;
        head.quaternion.copy(rotated);
      } else {
        head.quaternion.copy(relativeRotation);
      }
    }
  }

  dispose() {
    this.group.removeFromParent();
  }
}

function clamp(value: number, min = 0, max = 1) {
  return Math.max(min, Math.min(max, value));
}
