import {
  AmbientLight,
  CanvasTexture,
  Clock,
  EquirectangularReflectionMapping,
  PCFSoftShadowMap,
  PerspectiveCamera,
  Scene,
  sRGBEncoding,
  Texture,
  Vector2,
  Vector3,
  WebGLRenderer,
  WebGLRenderTarget,
} from "three";
import { CSM } from "three/examples/jsm/csm/CSM";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass";
import { ShaderPass } from "three/examples/jsm/postprocessing/ShaderPass";
import { GammaCorrectionShader } from "three/examples/jsm/shaders/GammaCorrectionShader";

import { DEFAULT_CONTROLS } from "../constants";
import { ControlsType } from "../Engine";
import { isSceneMessage } from "../scene/messages";
import { PostMessage, Transferable } from "../types";
import { OrbitControls } from "./controls/OrbitControls";
import { PlayerControls } from "./controls/PlayerControls";
import { RaycastControls } from "./controls/RaycastControls";
import { TransformControls } from "./controls/TransformControls";
import { FromRenderMessage, ToRenderMessage } from "./messages";
import { Players } from "./players/Players";
import { RenderScene } from "./scene/RenderScene";
import { ThreeOutlinePass } from "./three/ThreeOutlinePass";
import { deepDispose } from "./utils/deepDispose";

const CAMERA_NEAR = 0.01;
const CAMERA_FAR = 750;

export class RenderThread {
  #canvas: HTMLCanvasElement | OffscreenCanvas | null = null;
  postMessage: PostMessage<FromRenderMessage>;

  renderScene: RenderScene;
  scene = new Scene();

  renderer: WebGLRenderer | null = null;
  size = { width: 0, height: 0 };
  pixelRatio = 1;
  camera = new PerspectiveCamera(75, 1, CAMERA_NEAR, CAMERA_FAR);
  clock = new Clock();
  #animationFrame: number | null = null;

  outlinePass: ThreeOutlinePass | null = null;
  composer: EffectComposer | null = null;
  csm: CSM | null = null;

  transform = new TransformControls(this);
  orbit = new OrbitControls(this.camera, this.transform);
  player: PlayerControls;
  raycaster: RaycastControls;
  players = new Players(this.camera);

  controls: ControlsType = DEFAULT_CONTROLS;

  constructor(postMessage: PostMessage<FromRenderMessage>, canvas?: HTMLCanvasElement) {
    this.postMessage = postMessage;

    this.renderScene = new RenderScene(this.postMessage);
    this.player = new PlayerControls(this.camera, this.renderScene.root);

    if (canvas) {
      this.#canvas = canvas;
      this.init();
    }

    this.raycaster = new RaycastControls(
      this.camera,
      this.renderScene,
      this.postMessage,
      this.transform
    );

    this.scene.add(this.renderScene.root);
    this.scene.add(this.player.group);
    this.scene.add(this.players.group);
    this.scene.add(new AmbientLight(0xffffff, 0.2));

    this.camera.position.set(0, 4, 12);
    this.camera.lookAt(0, 0, 0);

    this.clock.start();
    this.render();

    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.transform.onmessage(event.data);
    this.player.onmessage(event.data);
    this.players.onmessage(event.data);

    if (this.controls === "orbit") {
      this.orbit.onmessage(event.data);
      this.raycaster.onmessage(event.data);
    }

    if (isSceneMessage(event.data)) {
      this.renderScene.onmessage(event.data);
    }

    const { subject, data } = event.data;

    switch (subject) {
      case "set_canvas": {
        this.#canvas = data;
        this.init();
        break;
      }

      case "set_size": {
        this.size = data;

        this.camera.aspect = data.width / data.height;
        this.camera.updateProjectionMatrix();

        this.renderer?.setSize(data.width, data.height, false);
        this.orbit.setSize(data.width, data.height);
        this.outlinePass?.setSize(data.width, data.height);
        this.composer?.setSize(data.width, data.height);
        break;
      }

      case "set_pixel_ratio": {
        this.pixelRatio = data;
        this.renderer?.setPixelRatio(data);
        break;
      }

      case "set_skybox": {
        this.loadSkybox(data.uri);
        break;
      }

      case "set_controls": {
        this.controls = data;

        if (data === "orbit") {
          this.player.group.visible = false;
        } else {
          this.player.group.visible = true;
        }
        break;
      }

      case "toggle_visuals": {
        this.renderScene.visualsEnabled = data;
        break;
      }

      case "destroy": {
        if (this.#animationFrame) cancelAnimationFrame(this.#animationFrame);
        if (this.composer) this.composer.dispose();
        if (this.renderer) this.renderer.dispose();
        if (this.scene.background instanceof Texture) this.scene.background.dispose();
        if (this.scene.environment instanceof Texture) this.scene.environment.dispose();
        this.renderScene.destroy();
        this.transform.destroy();
        this.orbit.destroy();
        deepDispose(this.scene);
        break;
      }
    }
  };

  async loadSkybox(uri: string | null) {
    // Clean up old skybox
    if (this.scene.background instanceof Texture) this.scene.background.dispose();
    if (this.scene.environment instanceof Texture) this.scene.environment.dispose();

    if (!uri) {
      this.scene.environment = null;
      this.scene.background = null;
      return;
    }

    // Load skybox
    const res = await fetch(uri);
    const blob = await res.blob();
    const bitmap = await createImageBitmap(blob, { imageOrientation: "flipY" });

    const texture = new CanvasTexture(bitmap);
    texture.mapping = EquirectangularReflectionMapping;
    texture.encoding = sRGBEncoding;
    texture.needsUpdate = true;

    // Set skybox
    this.scene.environment = texture;
    this.scene.background = texture;
  }

  init() {
    if (!this.#canvas) return;

    this.renderer = new WebGLRenderer({
      canvas: this.#canvas,
      antialias: true,
      powerPreference: "high-performance",
      preserveDrawingBuffer: true,
    });

    this.renderer.setPixelRatio(this.pixelRatio);
    this.renderer.setSize(this.size.width, this.size.height, false);
    this.renderer.outputEncoding = sRGBEncoding;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = PCFSoftShadowMap;

    if (process.env.NODE_ENV === "production") {
      this.renderer.debug.checkShaderErrors = false;
    }

    // Cascading shadow maps
    this.csm = new CSM({
      maxFar: 40,
      cascades: 2,
      lightIntensity: 0.5,
      lightDirection: new Vector3(0.2, -1, 0.4).normalize(),
      shadowMapSize: 2048,
      camera: this.camera,
      parent: this.scene,
      shadowBias: -0.00001,
    });
    this.csm.fade = true;
    this.csm.setupMaterial(RenderScene.DEFAULT_MATERIAL);
    this.renderScene.csm = this.csm;

    // Post-processing
    const renderPass = new RenderPass(this.scene, this.camera);
    const gammaCorrectionPass = new ShaderPass(GammaCorrectionShader);
    this.outlinePass = new ThreeOutlinePass(
      new Vector2(this.size.width, this.size.height),
      this.scene,
      this.camera
    );
    const target = new WebGLRenderTarget(this.size.width, this.size.height, {
      encoding: sRGBEncoding,
      samples: 8,
    });

    this.composer = new EffectComposer(this.renderer, target);
    this.composer.addPass(renderPass);
    this.composer.addPass(this.outlinePass);
    this.composer.addPass(gammaCorrectionPass);
  }

  render() {
    this.#animationFrame = requestAnimationFrame(() => this.render());
    const delta = this.clock.getDelta();

    if (this.controls === "player") {
      this.player.update(delta);
    } else {
      this.orbit.update();
    }

    if (this.size.width === 0 || this.size.height === 0) return;

    this.players.update(delta);
    this.csm?.update();
    this.composer?.render();
  }

  toMainThread(message: FromRenderMessage, transfer?: Transferable[]) {
    this.postMessage(message, transfer);
  }
}
