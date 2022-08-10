import {
  CubeTextureLoader,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { disposeTree } from "./utils/disposeTree";

export interface RenderManagerOptions {
  skyboxPath?: string;
}

const defaultOptions = {};

export class RenderManager {
  scene = new Scene();
  camera: PerspectiveCamera;
  renderer: WebGLRenderer;

  #animationFrameId: number | null = null;
  #canvasWidth = 0;
  #canvasHeight = 0;

  constructor(canvas: HTMLCanvasElement, options?: RenderManagerOptions) {
    const { skyboxPath } = Object.assign(defaultOptions, options);

    // Renderer
    this.renderer = new WebGLRenderer({
      canvas,
      antialias: true,
      preserveDrawingBuffer: true,
    });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setSize(canvas.width, canvas.height);
    this.renderer.outputEncoding = sRGBEncoding;
    this.renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this.renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this.scene.environment = environment;

    // Skybox
    if (skyboxPath) {
      this.scene.background = new CubeTextureLoader()
        .setPath(skyboxPath)
        .load(["right.bmp", "left.bmp", "top.bmp", "bottom.bmp", "front.bmp", "back.bmp"]);
    }

    // Camera
    this.camera = new PerspectiveCamera(75, canvas.width / canvas.height);
    this.camera.position.set(0, 0, 5);
    this.camera.lookAt(0, 0, 0);

    // Start rendering
    this.start();
  }

  start() {
    this.stop();
    this.#render();
  }

  stop() {
    if (this.#animationFrameId !== null) cancelAnimationFrame(this.#animationFrameId);
  }

  destroy() {
    // Stop rendering
    this.stop();
    // Dispose rendering context
    this.renderer.dispose();
    // Dispose all objects
    disposeTree(this.scene);
  }

  #render() {
    this.#animationFrameId = requestAnimationFrame(() => this.#render());
    if (!this.renderer || !this.camera) return;

    this.#updateCanvasSize();
    this.renderer.render(this.scene, this.camera);
  }

  #updateCanvasSize() {
    if (!this.renderer || !this.camera) return;

    const width = this.renderer.domElement.width;
    const height = this.renderer.domElement.height;

    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
  }
}
