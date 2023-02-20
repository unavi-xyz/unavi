import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import {
  AnimationAction,
  AnimationMixer,
  Box3,
  Camera,
  Group,
  Mesh,
  MeshBasicMaterial,
  Quaternion,
  Shape,
  ShapeGeometry,
  Vector3,
} from "three";
import { Font, FontLoader } from "three/examples/jsm/loaders/FontLoader";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { INPUT_ARRAY_ROUNDING, PLAYER_HEIGHT, SPRINT_SPEED, WALK_SPEED } from "../../constants";
import { deepDispose } from "../utils/deepDispose";
import { loadMixamoAnimation } from "./loadMixamoAnimation";

const MIN_WALK_DETECT = 0.1;
const MIN_WALK_SPEED = 2;
const SPRINT_CUTOFF = WALK_SPEED + 5;

const ANIMATION_NAME = {
  Falling: "Falling",
  Idle: "Idle",
  LeftWalk: "LeftWalk",
  RightWalk: "RightWalk",
  Sprint: "Sprint",
  Walk: "Walk",
} as const;

export class Avatar {
  readonly uri: string;

  #camera: Camera;
  group = new Group();
  vrm: VRM | null = null;

  mixer: AnimationMixer | null = null;
  animations = new Map<keyof typeof ANIMATION_NAME, AnimationAction>();
  weights = { left: 0, right: 0, walk: 0, sprint: 0, fall: 0 };

  #quat = new Quaternion();
  #quatb = new Quaternion();
  #vec3 = new Vector3();
  #prevHeadRotation = new Quaternion();

  #mode: "first-person" | "third-person" = "third-person";
  #animationsPath: string | null = null;
  grounded = true;
  sprinting = false;

  #name: string | null = null;
  #font: Font | null = null;
  #loadFontPromise: Promise<void> | null = null;
  #nameplate: Mesh | null = null;
  #height: number = PLAYER_HEIGHT;

  velocity = new Vector3();
  targetPosition = new Vector3();
  targetRotation = new Quaternion();

  inputPosition: Int16Array | null = null;

  static NAMEPLATE_TEXT_MATERIAL = new MeshBasicMaterial({ color: 0xffffff });
  static NAMEPLATE_BG_MATERIAL = new MeshBasicMaterial({ color: 0x010101 });

  constructor(uri: string, camera: Camera) {
    this.uri = uri;
    this.#camera = camera;

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

      const boundingBox = new Box3().setFromObject(vrm.scene);
      const size = boundingBox.getSize(this.#vec3);
      this.height = size.y;

      this.group.add(vrm.scene);

      this.loadAnimations(vrm);
    });
  }

  get height() {
    return this.#height;
  }

  set height(height: number) {
    this.#height = height;

    if (this.#nameplate) {
      this.#nameplate.position.y = height + 0.2;
    }
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

  get name() {
    return this.#name;
  }

  set name(name: string | null) {
    if (this.#name === name) return;
    this.#name = name;
    this.group.name = name ?? "";

    if (name) {
      if (!this.#loadFontPromise) this.#loadFontPromise = this.loadFont();
      this.#loadFontPromise.then(() => this.#createNameplate(name));
    }
  }

  async loadFont() {
    if (this.#font) return;
    const loader = new FontLoader();
    this.#font = await loader.loadAsync(new URL("./font.json", import.meta.url).href);
  }

  #createNameplate(text: string) {
    if (!this.#font) throw new Error("No font");

    // Remove old nametag
    if (this.#nameplate) {
      this.group.remove(this.#nameplate);
      this.#nameplate.traverse((object) => {
        if (object instanceof Mesh) object.geometry.dispose();
      });
      this.#nameplate = null;
    }

    // Create text
    const shapes = this.#font.generateShapes(text, 0.075);
    const textGeometry = new ShapeGeometry(shapes);

    // Center horizontally
    textGeometry.computeBoundingBox();
    if (!textGeometry.boundingBox) throw new Error("No bounding box");
    const xMid = -0.5 * (textGeometry.boundingBox.max.x - textGeometry.boundingBox.min.x);
    textGeometry.translate(xMid, 0, 0);

    this.#nameplate = new Mesh(textGeometry, Avatar.NAMEPLATE_TEXT_MATERIAL);
    this.#nameplate.position.y = this.#height + 0.2;
    this.#nameplate.rotation.y = Math.PI;

    this.group.add(this.#nameplate);

    // Create background
    const width = textGeometry.boundingBox.max.x - textGeometry.boundingBox.min.x + 0.15;
    const height = textGeometry.boundingBox.max.y - textGeometry.boundingBox.min.y + 0.06;
    const radius = height / 2;

    const shape = new Shape();
    shape.moveTo(0, radius);
    shape.lineTo(0, height - radius);
    shape.quadraticCurveTo(0, height, radius, height);
    shape.lineTo(width - radius, height);
    shape.quadraticCurveTo(width, height, width, height - radius);
    shape.lineTo(width, radius);
    shape.quadraticCurveTo(width, 0, width - radius, 0);
    shape.lineTo(radius, 0);
    shape.quadraticCurveTo(0, 0, 0, radius);

    const roundedRectangle = new ShapeGeometry(shape);

    const background = new Mesh(roundedRectangle, Avatar.NAMEPLATE_BG_MATERIAL);
    background.position.x = -width / 2;
    background.position.y = -height / 4;
    background.position.z = -0.001;

    this.#nameplate.add(background);
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
    const K = 1 - Math.pow(10e-8, delta);

    this.vrm.update(delta);

    if (this.inputPosition) {
      // Calculate velocity based on input
      const x = Atomics.load(this.inputPosition, 0) / INPUT_ARRAY_ROUNDING;
      const z = Atomics.load(this.inputPosition, 1) / INPUT_ARRAY_ROUNDING;

      // Input is relative to camera, so we need to rotate it
      this.#vec3
        .set(-x, 0, z)
        .applyQuaternion(this.#camera.quaternion)
        .multiplyScalar(this.sprinting ? SPRINT_SPEED : WALK_SPEED);

      this.velocity.lerp(this.#vec3, K);
    } else {
      // Calculate velocity based on position
      const deltaPos = this.#vec3
        .copy(this.group.position)
        .sub(this.targetPosition)
        .divideScalar(delta);

      this.velocity.lerp(deltaPos, K);
    }

    // Move body
    this.group.position.lerp(this.targetPosition, K);

    // Apply animations
    if (this.mixer) {
      const relativeVelocity = this.#vec3
        .copy(this.velocity)
        .applyQuaternion(this.group.getWorldQuaternion(this.#quat).invert());

      const leftVelocity = relativeVelocity.x > MIN_WALK_DETECT ? Math.abs(relativeVelocity.x) : 0;
      const rightVelocity = relativeVelocity.x < MIN_WALK_DETECT ? Math.abs(relativeVelocity.x) : 0;
      const horizontalVelocity = Math.max(leftVelocity, rightVelocity);
      const forwardVelocity =
        Math.abs(relativeVelocity.z) > MIN_WALK_DETECT ? Math.abs(relativeVelocity.z) : 0;
      const isBackwards = relativeVelocity.z < 0;

      this.weights.walk = clamp(
        clamp(
          forwardVelocity > MIN_WALK_SPEED && this.grounded
            ? this.weights.walk + delta * 8
            : this.weights.walk - delta * 8
        ) - this.weights.sprint
      );

      this.weights.sprint = clamp(
        (forwardVelocity > SPRINT_CUTOFF ||
          (forwardVelocity > WALK_SPEED * 0.75 && horizontalVelocity > WALK_SPEED * 0.75)) &&
          this.grounded
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
        if (leftWalk.isRunning() && this.weights.left === 0) leftWalk.reset();
        if (!leftWalk.isRunning() && this.weights.left > 0) leftWalk.play();
        leftWalk.setEffectiveWeight(this.weights.left);
      }

      if (rightWalk) {
        if (rightWalk.isRunning() && this.weights.right === 0) rightWalk.reset();
        if (!rightWalk.isRunning() && this.weights.right > 0) rightWalk.play();
        rightWalk.setEffectiveWeight(this.weights.right);
      }

      if (walk) {
        if (walk.isRunning() && this.weights.walk === 0 && this.weights.sprint === 0) walk.reset();
        if (!walk.isRunning() && this.weights.walk > 0) walk.play();
        walk.setEffectiveWeight(this.weights.walk);
        walk.setEffectiveTimeScale(isBackwards ? -1 : 1);
      }

      if (sprint) {
        if (sprint.isRunning() && this.weights.sprint === 0) sprint.reset();
        if (!sprint.isRunning() && this.weights.sprint > 0) sprint.play();
        sprint.setEffectiveWeight(this.weights.sprint);
        sprint.setEffectiveTimeScale(isBackwards ? -1 : 1);
      }

      if (fall) {
        if (fall.isRunning() && this.weights.fall === 0) fall.reset();
        if (!fall.isRunning() && this.weights.fall > 0) fall.play();
        fall.setEffectiveWeight(this.weights.fall);
      }

      const idleWeight = 1 - Object.values(this.weights).reduce((a, b) => a + b, 0);
      if (idle) {
        if (idle.isRunning() && idleWeight === 0) idle.reset();
        if (!idle.isRunning() && idleWeight > 0) idle.play();
        idle.setEffectiveWeight(idleWeight);
      }

      this.mixer.update(delta);
    }

    // Rotate body around Y axis
    this.group.quaternion.slerp(this.targetRotation, K);
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

      head.quaternion.copy(this.#prevHeadRotation);

      if (this.vrm.meta.metaVersion === "1") {
        // If vrm 1.0, rotate axis
        const rotated = this.#quatb.copy(relativeRotation);
        rotated.x = relativeRotation.z;
        rotated.z = -relativeRotation.x;
        head.quaternion.slerp(rotated, K);
      } else {
        head.quaternion.slerp(relativeRotation, K);
      }

      this.#prevHeadRotation.copy(head.quaternion);
    }

    // Update nameplate
    if (this.#nameplate) {
      // Hide if too far away
      this.#nameplate.visible = this.#camera.position.distanceTo(this.group.position) < 12;
      // Rotate to face camera
      this.#nameplate.lookAt(this.#camera.position);
    }
  }

  destroy() {
    this.group.removeFromParent();
    deepDispose(this.group);
  }
}

function clamp(value: number, min = 0, max = 1) {
  return Math.max(min, Math.min(max, value));
}
