import {
  AmbientLight,
  Clock,
  DirectionalLight,
  FogExp2,
  PCFSoftShadowMap,
  PerspectiveCamera,
  PMREMGenerator,
  Scene,
  sRGBEncoding,
  WebGLRenderer,
} from "three";

import { PostMessage } from "../types";
import { OrbitControlsPlugin } from "./plugins/OrbitControls/OrbitControlsPlugin";
import { OtherPlayersPlugin } from "./plugins/OtherPlayers/OtherPlayersPlugin";
import { PlayerPlugin } from "./plugins/PlayerPlugin";
import { RaycasterPlugin } from "./plugins/RaycasterPlugin";
import { TransformControlsPlugin } from "./plugins/TransformControls/TransformControlsPlugin";
import { SceneLoader } from "./SceneLoader/SceneLoader";
import { FromRenderMessage, Plugin, ToRenderMessage } from "./types";
import { disposeObject } from "./utils/disposeObject";
import { loadCubeTexture } from "./utils/loadCubeTexture";

const SHADOW_VIEW_DISTANCE = 50;

export type RenderWorkerOptions = {
  pixelRatio: number;
  canvasWidth: number;
  canvasHeight: number;
  avatarAnimationsPath?: string;
  avatarPath?: string;
  camera: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  skyboxPath?: string;
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
  #sun = new DirectionalLight(0xfff0db, 0.98);

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

    // Add sun
    this.#sun.castShadow = true;
    this.#sun.position.set(0, 200, 0);
    this.#scene.add(this.#sun);

    this.#sun.shadow.mapSize.width = 2048;
    this.#sun.shadow.mapSize.height = 2048;
    this.#sun.shadow.bias = -0.0005;
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.#plugins.forEach((plugin) => plugin.onmessage(event));
    this.#sceneLoader.onmessage(event);

    const { subject, data } = event.data;
    switch (subject) {
      case "set_canvas": {
        this.#canvas = data;
        break;
      }

      case "init": {
        this.init(data);
        break;
      }

      case "start": {
        this.start();
        break;
      }

      case "stop": {
        this.stop();
        break;
      }

      case "destroy": {
        this.destroy();
        break;
      }

      case "size": {
        this.#updateCanvasSize(data.width, data.height);
        break;
      }
    }
  };

  async init({
    pixelRatio,
    canvasWidth,
    canvasHeight,
    avatarAnimationsPath,
    avatarPath,
    camera,
    enableTransformControls = false,
    preserveDrawingBuffer = false,
    skyboxPath,
  }: RenderWorkerOptions) {
    if (!this.#canvas) throw new Error("Canvas not set");

    this.#plugins.push(
      new OtherPlayersPlugin(
        this.#scene,
        this.#postMessage,
        avatarPath,
        avatarAnimationsPath
      )
    );

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
    this.#renderer.shadowMap.type = PCFSoftShadowMap;

    // Lights
    const ambientLight = new AmbientLight(0xffffff, 0.02);
    this.#scene.add(ambientLight);

    // Fog
    this.#scene.fog = new FogExp2(0xeefaff, 0.004);

    // Camera
    this.#camera = new PerspectiveCamera(
      75,
      canvasWidth / canvasHeight,
      0.17,
      750
    );

    switch (camera) {
      case "orbit": {
        this.#plugins.push(
          new OrbitControlsPlugin(
            this.#camera,
            canvasWidth,
            canvasHeight,
            this.#pluginState
          )
        );
        break;
      }

      case "player": {
        const plugin = new PlayerPlugin(
          this.#camera,
          this.#postMessage,
          avatarPath,
          avatarAnimationsPath
        );
        this.#plugins.push(plugin);
        this.#scene.add(plugin.group);
        break;
      }
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

        this.#postMessage({ subject: "ready", data: null });
      });
    } else {
      this.#postMessage({ subject: "ready", data: null });
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

  #animate() {
    const delta = this.#clock.getDelta();
    this.#animationFrameId = requestAnimationFrame(() => this.#animate());
    if (!this.#renderer || !this.#camera) return;

    this.#sceneLoader.mixer.update(delta);
    this.#plugins.forEach((plugin) => plugin.animate && plugin.animate(delta));

    this.#updateShadowMap();

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

  #updateShadowMap() {
    if (!this.#camera) return;

    const { x, z } = this.#camera.position;

    this.#sun.shadow.camera.left = -SHADOW_VIEW_DISTANCE + x;
    this.#sun.shadow.camera.right = SHADOW_VIEW_DISTANCE + x;
    this.#sun.shadow.camera.top = -SHADOW_VIEW_DISTANCE - z;
    this.#sun.shadow.camera.bottom = SHADOW_VIEW_DISTANCE - z;

    this.#sun.shadow.camera.updateProjectionMatrix();
  }
}
