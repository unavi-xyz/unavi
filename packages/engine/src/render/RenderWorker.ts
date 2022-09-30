import {
  Clock,
  FogExp2,
  PerspectiveCamera,
  PMREMGenerator,
  Scene,
  sRGBEncoding,
  WebGLRenderer,
} from "three";

import { PostMessage } from "../types";
import { OrbitControlsPlugin } from "./plugins/OrbitControlsPlugin";
import { PlayerPlugin } from "./plugins/PlayerPlugin";
import { RaycasterPlugin } from "./plugins/RaycasterPlugin";
import { TransformControlsPlugin } from "./plugins/TransformControlsPlugin";
import { SceneLoader } from "./SceneLoader/SceneLoader";
import { FromRenderMessage, Plugin, ToRenderMessage } from "./types";
import { disposeObject } from "./utils/disposeObject";
import { loadCubeTexture } from "./utils/loadCubeTexture";

export type RenderWorkerOptions = {
  pixelRatio: number;
  canvasWidth: number;
  canvasHeight: number;
  camera: "orbit" | "player";
  skyboxPath?: string;
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
};

export type PluginState = {
  usingTransformControls: boolean;
};

/*
 * Renders the scene using Three.js.
 * Can only be run in a Web Worker if the browser supports OffscreenCanvas.
 */
export class RenderWorker {
  #sceneLoader: SceneLoader;

  #postMessage: PostMessage<FromRenderMessage>;
  #canvas: HTMLCanvasElement | OffscreenCanvas | undefined;
  #scene = new Scene();
  #renderer: WebGLRenderer | null = null;
  #camera: PerspectiveCamera | null = null;

  #animationFrameId: number | null = null;
  #canvasWidth = 0;
  #canvasHeight = 0;
  #clock = new Clock();

  #plugins: Plugin<ToRenderMessage>[] = [];
  #pluginState: PluginState = {
    usingTransformControls: false,
  };

  constructor(postMessage: PostMessage, canvas?: HTMLCanvasElement) {
    this.#canvas = canvas;
    this.#postMessage = postMessage;

    this.#sceneLoader = new SceneLoader(postMessage);
    this.#scene.add(this.#sceneLoader.root);
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.#plugins.forEach((plugin) => plugin.onmessage(event));
    this.#sceneLoader.onmessage(event);

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
      case "size":
        this.#updateCanvasSize(data.width, data.height);
        break;
      case "take_screenshot":
        this.takeScreenshot();
        break;
    }
  };

  async init({
    pixelRatio,
    canvasWidth,
    canvasHeight,
    camera,
    skyboxPath,
    enableTransformControls = false,
    preserveDrawingBuffer = false,
  }: RenderWorkerOptions) {
    if (!this.#canvas) throw new Error("Canvas not set");

    this.#canvasWidth = canvasWidth;
    this.#canvasHeight = canvasHeight;

    // Renderer
    this.#renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: pixelRatio > 1 ? false : true,
      powerPreference: "high-performance",
      preserveDrawingBuffer,
    });
    this.#renderer.setPixelRatio(pixelRatio);
    this.#renderer.setSize(canvasWidth, canvasHeight, false);
    this.#renderer.outputEncoding = sRGBEncoding;
    this.#renderer.shadowMap.enabled = true;

    // Fog
    this.#scene.fog = new FogExp2(0xeefaff, 0.005);

    // Camera
    this.#camera = new PerspectiveCamera(
      75,
      canvasWidth / canvasHeight,
      0.1,
      1000
    );

    // Camera
    switch (camera) {
      case "orbit":
        this.#plugins.push(
          new OrbitControlsPlugin(
            this.#camera,
            canvasWidth,
            canvasHeight,
            this.#pluginState
          )
        );
        break;
      case "player":
        this.#plugins.push(new PlayerPlugin(this.#camera));
        break;
      default:
        throw new Error(`Unknown camera: ${camera}`);
    }

    // Transform Controls
    if (enableTransformControls) {
      this.#plugins.unshift(
        new TransformControlsPlugin(
          this.#camera,
          this.#sceneLoader,
          this.#scene,
          this.#pluginState
        ),
        new RaycasterPlugin(
          this.#camera,
          this.#sceneLoader,
          this.#postMessage,
          this.#pluginState
        )
      );
    }

    // Skybox
    if (skyboxPath) {
      loadCubeTexture(skyboxPath).then((texture) => {
        texture.encoding = sRGBEncoding;

        this.#scene.background = texture;
        this.#scene.environment = texture;

        // Generate PMREM mipmaps
        if (!this.#renderer) throw new Error("Renderer not initialized");
        const premGenerator = new PMREMGenerator(this.#renderer);
        premGenerator.compileEquirectangularShader();

        setTimeout(() => {
          this.#postMessage({ subject: "ready", data: null });
        }, 500);
      });
    } else {
      // Ready for rendering
      // Add a delay, without it sometimes the scene glitches out and turns black, idk why
      // Still happens sometimes, but it's better
      // TODO: Figure out why this happens
      setTimeout(() => {
        this.#postMessage({ subject: "ready", data: null });
      }, 500);
    }
  }

  start() {
    this.stop();
    this.#clock.start();
    this.#animate();
  }

  stop() {
    this.#clock.stop();
    if (this.#animationFrameId !== null) {
      cancelAnimationFrame(this.#animationFrameId);
      this.#animationFrameId = null;
    }
  }

  destroy() {
    this.stop();
    this.#plugins.forEach((plugin) => plugin.destroy && plugin.destroy());
    disposeObject(this.#scene);
    this.#renderer?.dispose();
  }

  takeScreenshot() {
    if (!this.#renderer) throw new Error("Renderer not initialized");
    const data = this.#renderer.domElement.toDataURL("image/jpeg", 1);
    this.#postMessage({ subject: "screenshot", data });
  }

  #animate() {
    const delta = this.#clock.getDelta();
    this.#animationFrameId = requestAnimationFrame(() => this.#animate());
    if (!this.#renderer || !this.#camera) return;

    this.#sceneLoader.mixer.update(delta);
    this.#plugins.forEach((plugin) => plugin.animate && plugin.animate(delta));

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
