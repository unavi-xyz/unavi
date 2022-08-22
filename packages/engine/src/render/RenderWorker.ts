import {
  AnimationMixer,
  Box3,
  CubeTextureLoader,
  FogExp2,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  Vector3,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { PostMessage } from "../types";
import { disposeTree } from "../utils/disposeTree";
import { SceneLoader } from "./SceneLoader";
import { FromRenderMessage, LoadSceneData, RenderWorkerOptions, ToRenderMessage } from "./types";

export class RenderWorker {
  #postMessage: PostMessage<FromRenderMessage>;
  #canvas: HTMLCanvasElement | OffscreenCanvas | undefined;

  #scene = new Scene();
  #renderer: WebGLRenderer | null = null;
  #camera: PerspectiveCamera | null = null;
  #orbitControls: OrbitControls | null = null;
  #mixer: AnimationMixer | null = null;

  #animationFrameId: number | null = null;
  #canvasWidth = 0;
  #canvasHeight = 0;
  #lastTime = performance.now();

  constructor(postMessage: PostMessage, canvas?: HTMLCanvasElement) {
    this.#postMessage = postMessage;
    this.#canvas = canvas;
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
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
      default:
        throw new Error(`Unknown message subject: ${subject}`);
    }
  };

  init({ skyboxPath, controls }: RenderWorkerOptions) {
    if (!this.#canvas) throw new Error("Canvas not set");

    // Renderer
    this.#renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: true,
      alpha: true,
      preserveDrawingBuffer: true,
    });
    this.#renderer.setPixelRatio(window.devicePixelRatio);
    this.#renderer.setSize(this.#canvas.width, this.#canvas.height);
    this.#renderer.outputEncoding = sRGBEncoding;
    this.#renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this.#renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this.#scene.environment = environment;
    this.#scene.background = environment;

    // Fog
    this.#scene.fog = new FogExp2(0xeefaff, 0.005);

    // Skybox
    if (skyboxPath) {
      this.#scene.background = new CubeTextureLoader()
        .setPath(skyboxPath)
        .load(["right.bmp", "left.bmp", "top.bmp", "bottom.bmp", "front.bmp", "back.bmp"]);
    }

    // Camera
    this.#camera = new PerspectiveCamera(75, this.#canvas.width / this.#canvas.height, 0.1, 1000);

    // Controls
    if (controls === "orbit") {
      this.#orbitControls = new OrbitControls(this.#camera, this.#renderer.domElement);
      this.#orbitControls.enableDamping = true;
      this.#orbitControls.dampingFactor = 0.05;
      this.#camera.position.set(-1, 2, 5);
      this.#camera.lookAt(0, 0, 0);
    }

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
    disposeTree(this.#scene);
    this.#renderer?.dispose();
    this.#orbitControls?.dispose();
    this.#camera = null;
  }

  loadScene(data: LoadSceneData) {
    const parser = new SceneLoader();
    const { scene, animations } = parser.parse(data);
    this.#scene.add(scene);

    // Play animations
    if (animations.length > 0) {
      this.#mixer = new AnimationMixer(this.#scene);
      for (const animation of animations) {
        this.#mixer.clipAction(animation).play();
      }
    }

    if (this.#camera && this.#orbitControls) {
      // Center camera on scene
      const boundingBox = new Box3().setFromObject(scene);
      const size = boundingBox.getSize(new Vector3());
      const center = boundingBox.getCenter(new Vector3());

      if (size.x === 0) size.setX(1);
      if (size.y === 0) size.setY(1);
      if (size.z === 0) size.setZ(1);
      this.#camera.position.set(size.x, size.y, size.z * 2);

      this.#camera.lookAt(center);
      this.#orbitControls.target.copy(center);

      const min = boundingBox.min.z === 0 ? 1 : boundingBox.min.z;
      const max = boundingBox.max.z === 0 ? 1 : boundingBox.max.z;
      this.#camera.near = Math.abs(min) / 1000;
      this.#camera.far = Math.abs(max) * 100;
      this.#camera.far = Math.max(this.#camera.far, 50);
      this.#camera.updateProjectionMatrix();
    }
  }

  #animate() {
    this.#animationFrameId = requestAnimationFrame(() => this.#animate());
    if (!this.#renderer || !this.#camera) return;

    const time = performance.now();
    const delta = time - this.#lastTime;
    this.#lastTime = time;

    this.#updateCanvasSize();
    this.#mixer?.update(delta / 1000);
    this.#orbitControls?.update();
    this.#renderer.render(this.#scene, this.#camera);
  }

  #updateCanvasSize() {
    if (!this.#renderer || !this.#camera) return;

    const width = this.#renderer.domElement.width;
    const height = this.#renderer.domElement.height;

    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.#renderer.setSize(width, height);
    this.#camera.aspect = width / height;
    this.#camera.updateProjectionMatrix();
  }
}
