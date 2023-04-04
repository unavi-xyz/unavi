import {
  AmbientLight,
  BufferAttribute,
  BufferGeometry,
  CanvasTexture,
  Clock,
  EquirectangularReflectionMapping,
  LineBasicMaterial,
  LineSegments,
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
import * as THREE from "three";
import { CSM } from "three/examples/jsm/csm/CSM";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass";
import { ShaderPass } from "three/examples/jsm/postprocessing/ShaderPass";
import { GammaCorrectionShader } from "three/examples/jsm/shaders/GammaCorrectionShader";
import { acceleratedRaycast, computeBoundsTree, disposeBoundsTree } from "three-mesh-bvh";

import { DEFAULT_CONTROLS } from "../constants";
import { isSceneMessage } from "../scene/messages";
import { ControlsType, PostMessage, Transferable } from "../types";
import { OrbitControls } from "./controls/OrbitControls";
import { PlayerControls } from "./controls/PlayerControls";
import { RaycastControls } from "./controls/RaycastControls";
import { TransformControls } from "./controls/TransformControls";
import { FromRenderMessage, ToRenderMessage } from "./messages";
import { Players } from "./players/Players";
import { DEFAULT_MATERIAL } from "./scene/builders/PrimitiveBuilder";
import { RenderScene } from "./scene/RenderScene";
import { ThreeOutlinePass } from "./three/ThreeOutlinePass";
import { deepDispose } from "./utils/deepDispose";

const CAMERA_NEAR = 0.05;
const CAMERA_FAR = 500;

const SHADOW_CASCADES = 2;
const SHADOW_BIAS = -0.00007;

const TOTAL_LIGHT_INTENSITY = 1.2;
const AMBIENT_LIGHT_INTENSITY = 0.1;

THREE.BufferGeometry.prototype.computeBoundsTree = computeBoundsTree;
THREE.BufferGeometry.prototype.disposeBoundsTree = disposeBoundsTree;
THREE.Mesh.prototype.raycast = acceleratedRaycast;

/**
 * The render thread is responsible for rendering to the canvas.
 * The render loop runs at the browser's requestAnimationFrame rate.
 */
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
  #showColliderVisuals = false;
  #delta = 1;
  #raycastFrame = 0;
  #bvhFrame = 0;

  outlinePass: ThreeOutlinePass | null = null;
  composer: EffectComposer | null = null;
  csm: CSM | null = null;

  debugLines: LineSegments;
  #debugVertices = new Float32Array(0);
  #debugColors = new Float32Array(0);
  #debugFrame = 0;
  #debugInterval = 60;

  transform = new TransformControls(this);
  orbit = new OrbitControls(this.camera, this.transform);
  player: PlayerControls;
  raycaster = new RaycastControls(this);
  players = new Players(this.camera);

  controls: ControlsType = DEFAULT_CONTROLS;
  #prevCameraPosition = new Vector3(2, 4, 8);

  constructor(postMessage: PostMessage<FromRenderMessage>, canvas?: HTMLCanvasElement) {
    this.postMessage = postMessage;

    this.renderScene = new RenderScene();
    this.player = new PlayerControls(this.camera, this.renderScene.root);

    if (canvas) {
      this.#canvas = canvas;
      this.init();
    }

    this.scene.add(this.renderScene.root);
    this.scene.add(this.player.group);
    this.scene.add(this.players.group);
    this.scene.add(new AmbientLight(0xffffff, AMBIENT_LIGHT_INTENSITY));

    this.debugLines = new LineSegments(
      new BufferGeometry(),
      new LineBasicMaterial({ vertexColors: true })
    );
    this.debugLines.frustumCulled = false;
    this.scene.add(this.debugLines);

    this.clock.start();
    this.render();

    postMessage({ subject: "ready", data: null });
  }

  onmessage = (event: MessageEvent<ToRenderMessage>) => {
    this.transform.onmessage(event.data);
    this.player.onmessage(event.data);
    this.players.onmessage(event.data);
    this.raycaster.onmessage(event.data);

    if (this.controls === "orbit") this.orbit.onmessage(event.data);

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

      case "set_base_uri": {
        this.renderScene.baseURI = data;
        break;
      }

      case "get_stats": {
        const info = this.renderer?.info;
        if (!info) break;

        this.postMessage({
          subject: "stats",
          data: {
            memory: info.memory,
            render: info.render,
            fps: 1 / this.#delta,
          },
        });
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
        if (this.controls === data) break;
        this.controls = data;

        if (data === "orbit") {
          this.player.group.visible = false;
          this.camera.position.copy(this.#prevCameraPosition);
        } else {
          this.player.group.visible = true;
          this.#prevCameraPosition.copy(this.camera.position);
          this.transform.detach();
        }
        break;
      }

      case "toggle_collider_visuals": {
        this.#showColliderVisuals = data;
        this.debugLines.visible = data;
        break;
      }

      case "toggle_bvh_visuals": {
        this.renderScene.builders.node.setBvhVisuals(data);
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

      case "set_debug_buffers": {
        this.#debugVertices = data.vertices;
        this.#debugColors = data.colors;
        break;
      }

      case "toggle_animations": {
        this.renderScene.toggleAnimations(data);
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
    this.renderer.info.autoReset = false;

    if (process.env.NODE_ENV === "production") {
      this.renderer.debug.checkShaderErrors = false;
    }

    // Cascading shadow maps
    this.csm = new CSM({
      maxFar: 40,
      cascades: SHADOW_CASCADES,
      lightIntensity: (TOTAL_LIGHT_INTENSITY - AMBIENT_LIGHT_INTENSITY) / SHADOW_CASCADES,
      lightDirection: new Vector3(0.2, -1, 0.4).normalize(),
      shadowMapSize: 2048,
      camera: this.camera,
      parent: this.scene,
      shadowBias: SHADOW_BIAS,
    });
    this.csm.fade = true;
    this.csm.setupMaterial(DEFAULT_MATERIAL);
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
    this.#delta = delta;

    if (
      !this.composer ||
      !this.csm ||
      !this.renderer ||
      this.size.width === 0 ||
      this.size.height === 0
    )
      return;

    this.renderer.info.reset();

    if (this.#showColliderVisuals) {
      // Only update debug lines every 60 frames
      if (this.#debugFrame++ % this.#debugInterval === 0) {
        this.debugLines.geometry.setAttribute(
          "position",
          new BufferAttribute(this.#debugVertices, 3)
        );
        this.debugLines.geometry.setAttribute("color", new BufferAttribute(this.#debugColors, 4));

        // Adjust debug interval based on number vertices
        this.#debugInterval = Math.max(1, Math.floor(this.#debugVertices.length / 40000)) * 2;
      }
    }

    if (this.controls === "player") {
      this.player.update(delta);

      // Update raycast every 8 frames
      if (this.#raycastFrame++ % 8 === 0) {
        this.raycaster.update();
      }
    } else {
      this.orbit.update();
    }

    // Update bvh meshes every 300  frames
    if (this.#bvhFrame++ % 300 === 0) {
      this.renderScene.builders.node.regenerateMeshBVH();
    }

    this.renderScene.mixer.update(delta);
    this.players.update(delta);
    this.csm.update();

    this.composer.render();
  }

  toMainThread(message: FromRenderMessage, transfer?: Transferable[]) {
    this.postMessage(message, transfer);
  }
}
