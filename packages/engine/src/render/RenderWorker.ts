import {
  FogExp2,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { PostMessage } from "../types";
import { disposeObject } from "../utils/disposeObject";
import { SceneManager } from "./classes/SceneManager";
import { OrbitControlsPlugin } from "./plugins/OrbitControlsPlugin";
import { Plugin } from "./plugins/Plugin";
import { RaycasterPlugin } from "./plugins/RaycasterPlugin";
import { TransformControlsPlugin } from "./plugins/TransformControlsPlugin";
import { FromRenderMessage, LoadSceneData, ToRenderMessage } from "./types";
import { loadCubeTexture } from "./utils/loadCubeTexture";

export type RenderWorkerOptions = {
  pixelRatio: number;
  canvasWidth: number;
  canvasHeight: number;
  camera: "orbit" | "player";
  skyboxPath?: string;
  enableTransformControls?: boolean;
};

export type PluginState = {
  usingTransformControls: boolean;
};

export class RenderWorker {
  #sceneManager: SceneManager;

  #postMessage: PostMessage<FromRenderMessage>;
  #canvas: HTMLCanvasElement | OffscreenCanvas | undefined;

  #scene = new Scene();
  #renderer: WebGLRenderer | null = null;
  #camera: PerspectiveCamera | null = null;

  #animationFrameId: number | null = null;
  #canvasWidth = 0;
  #canvasHeight = 0;
  #lastTime = performance.now();

  #plugins: Plugin[] = [];
  #pluginState: PluginState = {
    usingTransformControls: false,
  };

  constructor(postMessage: PostMessage, canvas?: HTMLCanvasElement) {
    this.#canvas = canvas;
    this.#postMessage = postMessage;
    this.#sceneManager = new SceneManager(this.#postMessage);
    this.#scene.add(this.#sceneManager.root);
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    this.#plugins.forEach((plugin) => plugin.onmessage(event));
    this.#sceneManager.onmessage(event);

    const { subject, data } = event.data;
    switch (subject) {
      case "set_canvas":
        this.#canvas = data;
        break;
      case "init":
        this.init(data);
        break;
      case "start":
        this.start();
        break;
      case "stop":
        this.stop();
        break;
      case "destroy":
        this.destroy();
        break;
      case "load_scene":
        this.loadScene(data);
        break;
      case "size":
        this.#updateCanvasSize(data.width, data.height);
        break;
    }
  }

  async init({
    pixelRatio,
    canvasWidth,
    canvasHeight,
    camera,
    skyboxPath,
    enableTransformControls = false,
  }: RenderWorkerOptions) {
    if (!this.#canvas) throw new Error("Canvas not set");

    this.#canvasWidth = canvasWidth;
    this.#canvasHeight = canvasHeight;

    // Renderer
    this.#renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: pixelRatio > 1 ? false : true,
      powerPreference: "high-performance",
    });
    this.#renderer.setPixelRatio(pixelRatio);
    this.#renderer.setSize(canvasWidth, canvasHeight, false);
    this.#renderer.outputEncoding = sRGBEncoding;
    this.#renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this.#renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this.#scene.environment = environment;

    // Fog
    this.#scene.fog = new FogExp2(0xeefaff, 0.005);

    // Skybox
    if (skyboxPath)
      loadCubeTexture(skyboxPath).then((texture) => (this.#scene.background = texture));

    // Camera
    this.#camera = new PerspectiveCamera(75, canvasWidth / canvasHeight, 0.1, 1000);

    // Camera
    switch (camera) {
      case "orbit":
        this.#plugins.push(
          new OrbitControlsPlugin(this.#camera, canvasWidth, canvasHeight, this.#pluginState)
        );
        break;
      case "player":
        break;
      default:
        throw new Error(`Unknown camera: ${camera}`);
    }

    // Transform Controls
    if (enableTransformControls)
      this.#plugins.unshift(
        new TransformControlsPlugin(
          this.#camera,
          this.#sceneManager,
          this.#scene,
          this.#pluginState
        ),
        new RaycasterPlugin(this.#camera, this.#sceneManager, this.#postMessage, this.#pluginState)
      );

    // Ready
    this.#postMessage({ subject: "ready", data: null });
  }

  start() {
    this.stop();
    this.#animate();
  }

  stop() {
    if (this.#animationFrameId !== null) {
      cancelAnimationFrame(this.#animationFrameId);
      this.#animationFrameId = null;
    }
  }

  destroy() {
    this.stop();
    this.#plugins.forEach((plugin) => plugin.destroy());
    disposeObject(this.#scene);
    this.#renderer?.dispose();
  }

  loadScene(data: LoadSceneData) {
    // if (this.#gltf) {
    //   this.#scene.remove(this.#gltf);
    //   disposeObject(this.#gltf);
    //   this.#gltf = null;
    // }
    // const parser = new SceneLoader();
    // const { scene, animations } = parser.parse(data);
    // this.#gltf = scene;
    // this.#scene.add(this.#gltf);
    // // Play animations
    // if (animations.length > 0) {
    //   this.#mixer = new AnimationMixer(this.#gltf);
    //   for (const animation of animations) {
    //     this.#mixer.clipAction(animation).play();
    //   }
    // }
    // // Center camera on scene if orbit camera are enabled
    // if (this.#camera && this.#orbitControls) {
    //   const boundingBox = new Box3().setFromObject(scene);
    //   const size = boundingBox.getSize(new Vector3());
    //   const center = boundingBox.getCenter(new Vector3());
    //   if (size.x === 0) size.setX(1);
    //   if (size.y === 0) size.setY(1);
    //   if (size.z === 0) size.setZ(1);
    //   this.#camera.position.set(size.x, size.y, size.z * 2);
    //   this.#camera.lookAt(center);
    //   this.#orbitControls.target.copy(center);
    //   const min = boundingBox.min.z === 0 ? 1 : boundingBox.min.z;
    //   const max = boundingBox.max.z === 0 ? 1 : boundingBox.max.z;
    //   const near = Math.abs(min) / 100;
    //   const far = Math.max(50, Math.abs(max) * 100);
    //   this.#scene.fog = new Fog(0xeefaff, near, far);
    //   this.#camera.near = near;
    //   this.#camera.far = far;
    //   this.#camera.updateProjectionMatrix();
    // }
  }

  #animate() {
    this.#animationFrameId = requestAnimationFrame(() => this.#animate());
    if (!this.#renderer || !this.#camera) return;

    const time = performance.now();
    const delta = time - this.#lastTime;
    this.#lastTime = time;

    this.#plugins.forEach((plugin) => plugin.animate(delta));
    this.#renderer.render(this.#scene, this.#camera);
  }

  #updateCanvasSize(width: number, height: number) {
    if (!this.#renderer || !this.#camera) return;
    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.#renderer.setSize(width, height, false);
    this.#camera.aspect = width / height;
    this.#camera.updateProjectionMatrix();
  }
}
