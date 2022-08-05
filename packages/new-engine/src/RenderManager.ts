import {
  AnimationClip,
  AnimationMixer,
  Object3D,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import Stats from "three/examples/jsm/libs/stats.module";

import { LoadedGLTF } from "./gltf";
import { GLTFExporter } from "./gltf/GLTFExporter";
import { GLTFParser } from "./gltf/GLTFParser";
import { disposeTree } from "./utils/disposeTree";

interface RenderManagerOptions {
  canvas: HTMLCanvasElement;
  stats?: boolean;
  alpha?: boolean;
}

export class RenderManager {
  scene = new Scene();
  camera: PerspectiveCamera;
  renderer: WebGLRenderer;

  #canvasWidth = 0;
  #canvasHeight = 0;
  #mixer: AnimationMixer | null = null;
  #clips: AnimationClip[] = [];
  #parser: GLTFParser | null = null;

  #currentGltf: Object3D | null = null;
  #canvas: HTMLCanvasElement;
  #stats: Stats | null = null;

  constructor({ canvas, stats = false, alpha = false }: RenderManagerOptions) {
    // Renderer
    this.#canvas = canvas;
    this.renderer = new WebGLRenderer({
      canvas,
      antialias: true,
      alpha,
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

    // Camera
    this.camera = new PerspectiveCamera(75, canvas.width / canvas.height);
    this.camera.position.set(0, 0, 5);
    this.camera.lookAt(0, 0, 0);

    // Stats
    if (stats) {
      this.#stats = Stats();
      this.#stats.setMode(2);
      this.#stats.dom.id = "stats";
      document.body.appendChild(this.#stats.dom);
    }
  }

  #updateCanvasSize() {
    const width = this.#canvas.width;
    const height = this.#canvas.height;

    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
  }

  render(delta: number) {
    this.#updateCanvasSize();
    if (this.#mixer) this.#mixer.update(delta);

    this.renderer.render(this.scene, this.camera);

    if (this.#stats) this.#stats.update();
  }

  async createGLTF(data: LoadedGLTF) {
    // Clear previous gltf
    if (this.#currentGltf) {
      this.#currentGltf.removeFromParent();
      disposeTree(this.#currentGltf);
      this.#currentGltf = null;
    }

    // Parse gltf
    this.#parser = new GLTFParser(data);
    const { scene, animations } = await this.#parser.parse();

    // Animations
    const mixer = new AnimationMixer(scene);
    const actions = animations.map((clip) => mixer.clipAction(clip));
    this.#mixer = mixer;
    this.#clips = animations;

    // Add to scene
    this.#currentGltf = scene;
    this.scene.add(scene);

    return {
      scene,
      animations: actions,
    };
  }

  export() {
    const exporter = new GLTFExporter();
    return exporter.exportAsBinary(this.scene, this.#clips);
  }

  destroy() {
    // Dispose rendering context
    this.renderer.dispose();

    // Dispose all objects
    disposeTree(this.scene);

    // Remove stats
    if (this.#stats) document.body.removeChild(this.#stats.dom);
  }

  info() {
    return this.renderer.info;
  }
}
