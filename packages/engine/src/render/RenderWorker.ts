import {
  AnimationMixer,
  Box3,
  CubeTextureLoader,
  Fog,
  FogExp2,
  Object3D,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  Vector3,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { PostMessage } from "../types";
import { disposeTree } from "../utils/disposeTree";
import {
  FakePointerEvent,
  FakeWheelEvent,
  OrbitControls,
  PointerCaptureEvent,
} from "./OrbitControls";
import { SceneLoader } from "./SceneLoader";
import { FromRenderMessage, LoadSceneData, RenderWorkerOptions, ToRenderMessage } from "./types";

export class RenderWorker {
  #postMessage: PostMessage<FromRenderMessage>;
  #canvas: HTMLCanvasElement | OffscreenCanvas | undefined;

  #scene = new Scene();
  #gltf: Object3D | null = null;
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
      case "size":
        this.#updateCanvasSize(data.width, data.height);
        break;
      case "pointermove":
        const pointerMoveEvent: FakePointerEvent = new CustomEvent("pointermove", {
          detail: data,
        });
        this.#orbitControls?.domElement.dispatchEvent(pointerMoveEvent);
        break;
      case "pointerdown":
        const pointerDownEvent: FakePointerEvent = new CustomEvent("pointerdown", { detail: data });
        this.#orbitControls?.domElement.dispatchEvent(pointerDownEvent);
        break;
      case "pointerup":
        const pointerUpEvent: FakePointerEvent = new CustomEvent("pointerup", { detail: data });
        this.#orbitControls?.domElement.dispatchEvent(pointerUpEvent);
        break;
      case "pointercancel":
        const pointerCancelEvent: FakePointerEvent = new CustomEvent("pointercancel", {
          detail: data,
        });
        this.#orbitControls?.domElement.dispatchEvent(pointerCancelEvent);
        break;
      case "wheel":
        const wheelEvent: FakeWheelEvent = new CustomEvent("wheel", { detail: data });
        this.#orbitControls?.domElement.dispatchEvent(wheelEvent);
        break;
      default:
        throw new Error(`Unknown message subject: ${subject}`);
    }
  };

  init({ skyboxPath, controls, pixelRatio, canvasWidth, canvasHeight }: RenderWorkerOptions) {
    if (!this.#canvas) throw new Error("Canvas not set");

    this.#canvasWidth = canvasWidth;
    this.#canvasHeight = canvasHeight;

    // Renderer
    this.#renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: true,
      alpha: true,
      preserveDrawingBuffer: true,
    });
    if (pixelRatio !== undefined) this.#renderer.setPixelRatio(pixelRatio);
    this.#renderer.setSize(canvasWidth, canvasHeight, false);
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
    this.#camera = new PerspectiveCamera(75, canvasWidth / canvasHeight, 0.1, 1000);

    // Controls
    if (controls === "orbit") {
      const target = new EventTarget();
      //@ts-ignore
      target.addEventListener("setPointerCapture", (e: PointerCaptureEvent) => {
        this.#postMessage({ subject: "setPointerCapture", data: e.detail });
      });
      //@ts-ignore
      target.addEventListener("releasePointerCapture", (e: PointerCaptureEvent) => {
        this.#postMessage({ subject: "releasePointerCapture", data: e.detail });
      });

      this.#orbitControls = new OrbitControls(this.#camera, target, canvasWidth, canvasHeight);
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
    if (this.#gltf) {
      this.#scene.remove(this.#gltf);
      disposeTree(this.#gltf);
      this.#gltf = null;
    }

    const parser = new SceneLoader();
    const { scene, animations } = parser.parse(data);

    this.#gltf = scene;
    this.#scene.add(this.#gltf);

    // Play animations
    if (animations.length > 0) {
      this.#mixer = new AnimationMixer(this.#gltf);
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
      const near = Math.abs(min) / 100;
      const far = Math.max(50, Math.abs(max) * 100);
      this.#scene.fog = new Fog(0xeefaff, near, far);
      this.#camera.near = near;
      this.#camera.far = far;
      this.#camera.updateProjectionMatrix();
    }
  }

  #animate() {
    this.#animationFrameId = requestAnimationFrame(() => this.#animate());
    if (!this.#renderer || !this.#camera) return;

    const time = performance.now();
    const delta = time - this.#lastTime;
    this.#lastTime = time;

    this.#mixer?.update(delta / 1000);
    this.#orbitControls?.update();
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
