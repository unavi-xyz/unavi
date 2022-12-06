import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import {
  AnimationAction,
  AnimationClip,
  AnimationMixer,
  BoxGeometry,
  Camera,
  Group,
  LoopPingPong,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  PerspectiveCamera,
  Quaternion,
  Shape,
  ShapeGeometry,
  Vector3,
  WebGLRenderer,
} from "three";
import { FontLoader } from "three/examples/jsm/loaders/FontLoader";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { PLAYER_HEIGHT, PLAYER_RADIUS } from "../../../constants";
import { PostMessage } from "../../../types";
import { toHex } from "../../../utils/toHex";
import { ObjectQueue } from "../../SceneLoader/ObjectQueue";
import { FromRenderMessage } from "../../types";
import { disposeObject } from "../../utils/disposeObject";
import { loadMixamoAnimation } from "./loadMixamoAnimation";
import { AnimationName } from "./types";

const LERP_FACTOR = 0.000001;

export class PlayerAvatar {
  readonly playerId: number;
  readonly group = new Group();

  isFalling = false;
  isFirstPerson = false;

  vrm: VRM | null = null;
  #camera?: PerspectiveCamera;
  #sceneCamera: Camera;
  #nameplate: Mesh | null = null;

  #defaultAvatarPath?: string;
  #avatarAnimationsPath?: string;
  #mixer: AnimationMixer | null = null;
  #actions = new Map<AnimationName, AnimationAction>();
  #postMessage: PostMessage<FromRenderMessage>;

  #fallWeight = 0;
  #leftWeight = 0;
  #rightWeight = 0;
  #forwardWeight = 0;
  #sprintWeight = 0;

  #averageVelocity = new Vector3();
  #headRotation = new Quaternion();
  #prevPosition = new Vector3();
  #targetPosition = new Vector3();
  #targetRotation = new Quaternion();
  #tempQuat = new Quaternion();
  #tempQuat2 = new Quaternion();
  #velocity = new Vector3();

  #loader = new GLTFLoader();
  #queue: ObjectQueue;

  constructor(
    playerId: number,
    avatar: string | null,
    postMessage: PostMessage<FromRenderMessage>,
    sceneCamera: Camera,
    renderer: WebGLRenderer,
    defaultAvatarPath?: string,
    avatarAnimationsPath?: string,
    camera?: PerspectiveCamera
  ) {
    this.playerId = playerId;
    this.#postMessage = postMessage;
    this.#defaultAvatarPath = defaultAvatarPath;
    this.#avatarAnimationsPath = avatarAnimationsPath;
    this.#camera = camera;
    this.#sceneCamera = sceneCamera;
    this.#queue = new ObjectQueue(renderer, sceneCamera);

    this.#loader.register((parser) => new VRMLoaderPlugin(parser));

    this.#loadModel(avatar ?? defaultAvatarPath, avatarAnimationsPath).catch(
      (error) => {
        console.error(error);
        console.error(`🚨 Failed to load ${this.playerId}'s avatar`);
      }
    );
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

    // Remove previous VRM
    if (this.vrm) {
      this.vrm.scene.removeFromParent();
      VRMUtils.deepDispose(this.vrm.scene);
      this.vrm = null;
    }

    // Remove previous mixer
    if (this.#mixer) {
      this.#mixer.stopAllAction();
      this.#actions.clear();
      this.#mixer = null;
    }

    // Enable first-person view if it's the user
    if (this.#camera && vrm.firstPerson) {
      vrm.firstPerson.setup();
      this.#camera.layers.enable(vrm.firstPerson.firstPersonOnlyLayer);
      this.#camera.layers.disable(vrm.firstPerson.thirdPersonOnlyLayer);
    }

    // Process VRM
    VRMUtils.removeUnnecessaryVertices(vrm.scene);
    VRMUtils.removeUnnecessaryJoints(vrm.scene);
    VRMUtils.rotateVRM0(vrm);

    vrm.scene.rotation.y += Math.PI;

    vrm.scene.traverse((object) => {
      if (object instanceof Mesh) object.castShadow = true;
    });

    // Set VRM model
    this.vrm = vrm;

    // Add scene to queue
    this.#queue.add(this.vrm.scene, this.group, true);

    if (avatarAnimationsPath) {
      // Create mixer
      this.#mixer = new AnimationMixer(vrm.scene);

      // Load animations
      const animations = new Map<AnimationName, AnimationClip>();

      const clipPromises = Object.values(AnimationName).map(async (name) => {
        if (!this.vrm) throw new Error("VRM not loaded");
        if (!this.#mixer) throw new Error("Mixer not created");

        const path = `${avatarAnimationsPath}${name}.fbx`;

        try {
          const clips = await loadMixamoAnimation(path, this.vrm);
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
      this.#actions.get(AnimationName.Sprint)?.play();

      this.#actions
        .get(AnimationName.Falling)
        ?.play()
        .setLoop(LoopPingPong, Infinity);
    }

    if (this.playerId === -1) console.info(`💃 Loaded your avatar`);
    else console.info(`💃 Loaded ${toHex(this.playerId)}'s avatar`);

    this.#postMessage({
      subject: "player_loaded",
      data: this.playerId,
    });
  }

  async setName(name: string) {
    // Remove previous nameplate
    if (this.#nameplate) disposeObject(this.#nameplate);

    // Create text
    const loader = new FontLoader();
    const font = await loader.loadAsync(
      new URL("./font.json", import.meta.url).href
    );

    const shapes = font.generateShapes(name, 0.075);
    const geometry = new ShapeGeometry(shapes);

    // Center horizontally
    geometry.computeBoundingBox();
    if (!geometry.boundingBox) throw new Error("No bounding box");
    const xMid =
      -0.5 * (geometry.boundingBox.max.x - geometry.boundingBox.min.x);
    geometry.translate(xMid, 0, 0);

    const material = new MeshBasicMaterial();

    const mesh = new Mesh(geometry, material);
    mesh.position.y = PLAYER_HEIGHT - 0.25;
    mesh.rotation.y = Math.PI;

    this.group.add(mesh);
    this.#nameplate = mesh;

    // Create background
    const width =
      geometry.boundingBox.max.x - geometry.boundingBox.min.x + 0.15;
    const height =
      geometry.boundingBox.max.y - geometry.boundingBox.min.y + 0.06;
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

    const background = new Mesh(
      roundedRectangle,
      new MeshBasicMaterial({ color: 0x010101 })
    );
    background.position.x = -width / 2;
    background.position.y = -height / 4;
    background.position.z = -0.0001;

    this.#nameplate.add(background);
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

  setFirstPerson(firstPerson: boolean) {
    if (!this.vrm?.firstPerson) return;
    if (!this.#camera) throw new Error("Camera not set");

    this.isFirstPerson = firstPerson;

    // Enable first-person view if it's the user
    if (firstPerson) {
      this.vrm.firstPerson.setup();
      this.#camera.layers.enable(this.vrm.firstPerson.firstPersonOnlyLayer);
      this.#camera.layers.disable(this.vrm.firstPerson.thirdPersonOnlyLayer);
    } else {
      this.#camera.layers.disable(this.vrm.firstPerson.firstPersonOnlyLayer);
      this.#camera.layers.enable(this.vrm.firstPerson.thirdPersonOnlyLayer);
    }
  }

  animate(delta: number) {
    const K = 1 - Math.pow(LERP_FACTOR, delta);

    // Load queue
    this.#queue.update();

    // Apply location to group if not user
    if (!this.#camera || !this.vrm) {
      this.group.position.lerp(this.#targetPosition, K);
      this.group.quaternion.slerp(this.#targetRotation, K);
    }

    // Only rotate Y axis
    this.group.quaternion.x = 0;
    this.group.quaternion.z = 0;
    this.group.quaternion.normalize();

    // Get relative rotation for head
    const relativeRotation = this.#tempQuat2.copy(this.#targetRotation);
    relativeRotation.premultiply(
      this.#tempQuat.copy(this.group.quaternion).invert()
    );

    // Rotate head
    this.#headRotation.slerp(relativeRotation, K);

    // Don't rotate Y axis
    this.#headRotation.y = 0;
    this.#headRotation.normalize();

    // Calculate velocity relative to player rotation
    this.#velocity
      .copy(this.#prevPosition)
      .sub(this.group.position)
      .divideScalar(delta)
      .applyQuaternion(this.#tempQuat.copy(this.group.quaternion).invert());

    this.#prevPosition.copy(this.group.position);

    const velocity = this.#averageVelocity.lerp(this.#velocity, K);

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
    const leftVelocity = velocity.x > 0 ? velocity.x : 0;
    const rightVelocity = velocity.x < 0 ? -velocity.x : 0;
    const forwardVelocity = Math.abs(velocity.z);
    const isBackwards = velocity.z < 0;

    this.#leftWeight = clamp(
      leftVelocity > 1 && !this.isFalling
        ? this.#leftWeight + delta * 3
        : this.#leftWeight - delta * 6
    );

    this.#rightWeight = clamp(
      rightVelocity > 1 && !this.isFalling
        ? this.#rightWeight + delta * 3
        : this.#rightWeight - delta * 6
    );

    this.#sprintWeight = clamp(
      forwardVelocity > 4 && !this.isFalling
        ? this.#sprintWeight + delta * 6
        : this.#sprintWeight - delta * 4
    );

    this.#forwardWeight = clamp(
      forwardVelocity > 1 && !this.isFalling
        ? this.#forwardWeight + delta * 6
        : this.#forwardWeight - delta * 8
    );

    this.#actions
      .get(AnimationName.LeftWalk)
      ?.setEffectiveWeight(this.#leftWeight);

    this.#actions
      .get(AnimationName.RightWalk)
      ?.setEffectiveWeight(this.#rightWeight);

    this.#actions
      .get(AnimationName.Walk)
      ?.setEffectiveWeight(this.#forwardWeight - this.#sprintWeight)
      .setEffectiveTimeScale(isBackwards ? -1 : 1);

    this.#actions
      .get(AnimationName.Sprint)
      ?.setEffectiveWeight(this.#sprintWeight)
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
    if (this.vrm) this.vrm.update(delta);

    // Rotate head
    if (this.vrm?.meta.metaVersion === "1") {
      // If vrm 1.0, rotate axis
      const rotated = this.#tempQuat.copy(this.#headRotation);
      rotated.x = this.#headRotation.z;
      rotated.z = -this.#headRotation.x;

      this.vrm.humanoid.humanBones.head.node.quaternion.multiply(rotated);
    } else {
      this.vrm?.humanoid.humanBones.head.node.quaternion.multiply(
        this.#headRotation
      );
    }

    // Update nameplate
    if (this.#nameplate) {
      // Hide if too far away
      this.#nameplate.visible =
        this.#sceneCamera.position.distanceTo(this.group.position) < 10;

      // Rotate to face camera
      this.#nameplate.lookAt(this.#sceneCamera.position);
    }
  }

  destroy() {
    if (this.vrm) {
      this.vrm.scene.removeFromParent();
      VRMUtils.deepDispose(this.vrm.scene);
      this.vrm = null;
    }

    disposeObject(this.group);
  }
}

function clamp(value: number, min = 0, max = 1) {
  return Math.max(Math.min(value, max), min);
}
