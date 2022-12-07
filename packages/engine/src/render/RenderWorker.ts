import {
  Clock,
  Fog,
  PCFSoftShadowMap,
  PerspectiveCamera,
  PMREMGenerator,
  Scene,
  sRGBEncoding,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";
import { CSM } from "three/examples/jsm/csm/CSM";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer";
import { type OutlinePass as IOutlinePass } from "three/examples/jsm/postprocessing/OutlinePass";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass";
import { ShaderPass } from "three/examples/jsm/postprocessing/ShaderPass";
import { GammaCorrectionShader } from "three/examples/jsm/shaders/GammaCorrectionShader";

import { PostMessage } from "../types";
import { OrbitControlsPlugin } from "./plugins/OrbitControls/OrbitControlsPlugin";
import { OtherPlayersPlugin } from "./plugins/OtherPlayers/OtherPlayersPlugin";
import { PlayerPlugin } from "./plugins/PlayerPlugin";
import { RaycasterPlugin } from "./plugins/RaycasterPlugin";
import { OutlinePass } from "./plugins/TransformControls/OutlinePass";
import { TransformControlsPlugin } from "./plugins/TransformControls/TransformControlsPlugin";
import { RenderPlugin } from "./plugins/types";
import { defaultMaterial } from "./SceneLoader/constants";
import { SceneLoader } from "./SceneLoader/SceneLoader";
import { FromRenderMessage, ToRenderMessage } from "./types";
import { disposeObject } from "./utils/disposeObject";
import { loadCubeTexture } from "./utils/loadCubeTexture";

const CAMERA_FAR = 750;

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
  renderer: WebGLRenderer | null = null;
  camera: PerspectiveCamera | null = null;
  #sceneLoader: SceneLoader | null = null;

  #postMessage: PostMessage<FromRenderMessage>;
  #canvas: HTMLCanvasElement | OffscreenCanvas | undefined;
  #scene = new Scene();
  #composer: EffectComposer | null = null;
  outlinePass: IOutlinePass | null = null;

  #animationFrameId: number | null = null;
  #clock = new Clock();

  csm: CSM | null = null;

  #plugins: RenderPlugin[] = [];
  #pluginState: PluginState = {
    usingTransformControls: false,
  };

  constructor(postMessage: PostMessage, canvas?: HTMLCanvasElement) {
    this.#canvas = canvas;
    this.#postMessage = postMessage;
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.#plugins.forEach((plugin) => plugin.onmessage(event));
    this.#sceneLoader?.onmessage(event);

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

    // Renderer
    this.renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: pixelRatio > 1 ? false : true,
      powerPreference: "high-performance",
      preserveDrawingBuffer,
    });
    this.renderer.setPixelRatio(pixelRatio);
    this.renderer.setSize(canvasWidth, canvasHeight, false);
    this.renderer.outputEncoding = sRGBEncoding;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = PCFSoftShadowMap;

    // Fog
    this.#scene.fog = new Fog(0xcfcfcf, CAMERA_FAR / 2, CAMERA_FAR);

    // Camera
    this.camera = new PerspectiveCamera(
      75,
      canvasWidth / canvasHeight,
      0.1,
      CAMERA_FAR
    );

    switch (camera) {
      case "orbit": {
        this.#plugins.push(
          new OrbitControlsPlugin(
            this.camera,
            canvasWidth,
            canvasHeight,
            this.#pluginState
          )
        );
        break;
      }

      case "player": {
        const plugin = new PlayerPlugin(
          this.camera,
          this.#postMessage,
          this.renderer,
          avatarPath,
          avatarAnimationsPath
        );
        this.#plugins.push(plugin);
        this.#scene.add(plugin.group);
        break;
      }
    }

    // Other players
    this.#plugins.push(
      new OtherPlayersPlugin(
        this.#scene,
        this.#postMessage,
        this.renderer,
        this.camera,
        avatarPath,
        avatarAnimationsPath
      )
    );

    // Scene loader
    this.#sceneLoader = new SceneLoader(this.#postMessage, this);
    this.#scene.add(this.#sceneLoader.root);

    // Cascading shadow maps
    const cascades = 3;
    this.csm = new CSM({
      lightIntensity: 1 / cascades,
      maxFar: 75,
      cascades,
      lightDirection: new Vector3(0.2, -1, 0.4).normalize(),
      shadowMapSize: 2048,
      shadowBias: -0.0001,
      camera: this.camera,
      parent: this.#scene,
    });

    this.csm.fade = true;
    this.csm.setupMaterial(defaultMaterial);

    // Transform Controls
    if (enableTransformControls) {
      this.#plugins.unshift(
        new TransformControlsPlugin(
          this.camera,
          this.#sceneLoader,
          this.#scene,
          this.#pluginState,
          this
        ),
        new RaycasterPlugin(
          this.camera,
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
        if (!this.renderer) throw new Error("Renderer not initialized");
        const premGenerator = new PMREMGenerator(this.renderer);
        premGenerator.compileEquirectangularShader();

        this.#postMessage({ subject: "ready", data: null });
      });
    } else {
      this.#postMessage({ subject: "ready", data: null });
    }

    // Post-processing
    const renderPass = new RenderPass(this.#scene, this.camera);
    const gammaCorrectionPass = new ShaderPass(GammaCorrectionShader);

    // @ts-ignore
    this.outlinePass = new OutlinePass(
      new Vector2(canvasWidth, canvasHeight),
      this.#scene,
      this.camera
    ) as IOutlinePass;

    this.#composer = new EffectComposer(this.renderer);
    this.#composer.addPass(renderPass);
    this.#composer.addPass(this.outlinePass);
    this.#composer.addPass(gammaCorrectionPass);
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
    this.renderer?.dispose();
    this.#composer?.reset();
    this.outlinePass?.dispose();
  }

  #animate() {
    const delta = this.#clock.getDelta();
    this.#animationFrameId = requestAnimationFrame(() => this.#animate());

    if (!this.renderer || !this.camera || !this.#composer) return;

    this.#plugins.forEach((plugin) => {
      if (plugin.update) plugin.update(delta);
    });

    this.#sceneLoader?.update(delta);
    this.csm?.update();

    this.#composer.render();
  }

  #updateCanvasSize(width: number, height: number) {
    this.renderer?.setSize(width, height, false);

    if (this.camera) {
      this.camera.aspect = width / height;
      this.camera.updateProjectionMatrix();
    }

    this.#composer?.setSize(width, height);
    this.outlinePass?.setSize(width, height);
  }
}
