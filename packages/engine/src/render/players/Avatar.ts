import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import { AnimationAction, AnimationMixer, Group, Mesh, Quaternion, Vector3 } from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { loadMixamoAnimation } from "./loadMixamoAnimation";

const MAX_WALK_SPEED = 25;
const MAX_SPRINT_SPEED = 40;
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

  animations = new Map<keyof typeof ANIMATION_NAME, AnimationAction>();
  mixer: AnimationMixer | null = null;

  #quat = new Quaternion();
  #quatb = new Quaternion();
  #vec3 = new Vector3();

  #mode: "first-person" | "third-person" = "third-person";
  #animationsPath: string | null = null;

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
      const isBackwards = relativeVelocity.z < 0;

      const fallingWeight = normalizeWeight(Math.abs(relativeVelocity.y), 50, 20);
      const leftWeight = normalizeWeight(leftVelocity, MAX_WALK_SPEED * 0.75) - fallingWeight;
      const rightWeight = normalizeWeight(rightVelocity, MAX_WALK_SPEED * 0.75) - fallingWeight;
      const sprintWeight =
        normalizeWeight(forwardVelocity, MAX_SPRINT_SPEED * 0.75, SPRINT_CUTOFF) - fallingWeight;
      const walkWeight = normalizeWeight(forwardVelocity, MAX_WALK_SPEED * 0.75) - sprintWeight;
      const idleWeight = 1 - leftWeight - rightWeight - walkWeight - sprintWeight - fallingWeight;

      const leftWalk = this.animations.get(ANIMATION_NAME.LeftWalk);
      const rightWalk = this.animations.get(ANIMATION_NAME.RightWalk);
      const walk = this.animations.get(ANIMATION_NAME.Walk);
      const sprint = this.animations.get(ANIMATION_NAME.Sprint);
      const falling = this.animations.get(ANIMATION_NAME.Falling);
      const idle = this.animations.get(ANIMATION_NAME.Idle);

      if (leftWalk) {
        if (leftWalk.isRunning() && leftWeight < 0.1) leftWalk.stop();
        if (!leftWalk.isRunning() && leftWeight > 0.1) leftWalk.play();
        leftWalk.setEffectiveWeight(leftWeight);
      }

      if (rightWalk) {
        if (rightWalk.isRunning() && rightWeight < 0.1) rightWalk.stop();
        if (!rightWalk.isRunning() && rightWeight > 0.1) rightWalk.play();
        rightWalk.setEffectiveWeight(rightWeight);
      }

      if (walk) {
        if (walk.isRunning() && walkWeight < 0.1 && sprintWeight === 0) walk.stop();
        if (!walk.isRunning() && walkWeight > 0.1) walk.play();
        walk.setEffectiveWeight(walkWeight);
        walk.setEffectiveTimeScale(isBackwards ? -1 : 1);
      }

      if (sprint) {
        if (sprint.isRunning() && sprintWeight < 0.1) sprint.stop();
        if (!sprint.isRunning() && sprintWeight > 0.1) sprint.play();
        sprint.setEffectiveWeight(sprintWeight);
        sprint.setEffectiveTimeScale(isBackwards ? -1 : 1);
      }

      if (falling) {
        if (falling.isRunning() && fallingWeight === 0) falling.stop();
        if (!falling.isRunning() && fallingWeight > 0) falling.play();
        falling.setEffectiveWeight(fallingWeight);
      }

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

function normalizeWeight(weight: number, max: number, min = 0) {
  return Math.round(Math.max(0, Math.min(1, (weight - min) / (max - min))) * 100) / 100;
}
