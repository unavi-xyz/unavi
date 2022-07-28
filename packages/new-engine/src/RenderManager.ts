import {
  AnimationMixer,
  Box3,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  SkinnedMesh,
  Vector3,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import Stats from "three/examples/jsm/libs/stats.module";

import { LoadedGLTF } from "./gltf";
import { GLTFExporter } from "./gltf/GLTFExporter";
import { GLTFParser } from "./gltf/GLTFParser";
import { disposeTree } from "./utils/disposeTree";

export class RenderManager {
  #scene = new Scene();
  #canvasWidth = 0;
  #canvasHeight = 0;
  #mixers = new Map<string, AnimationMixer>();
  #parser: GLTFParser | null = null;

  #currentGltf: Object3D | null = null;
  #orbitControls: OrbitControls;
  #camera: PerspectiveCamera;
  #renderer: WebGLRenderer;
  #canvas: HTMLCanvasElement;
  #stats: Stats;

  constructor(canvas: HTMLCanvasElement) {
    this.#canvas = canvas;
    this.#renderer = new WebGLRenderer({ canvas, antialias: true, alpha: true });
    this.#renderer.setPixelRatio(window.devicePixelRatio);
    this.#renderer.setSize(canvas.width, canvas.height);
    this.#renderer.outputEncoding = sRGBEncoding;
    this.#renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this.#renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this.#scene.environment = environment;

    // Camera
    this.#camera = new PerspectiveCamera(75, canvas.width / canvas.height);

    // Stats
    this.#stats = Stats();
    this.#stats.setMode(2);
    this.#stats.dom.id = "stats";
    document.body.appendChild(this.#stats.dom);

    // Controls
    this.#orbitControls = new OrbitControls(this.#camera, canvas);
  }

  #updateCanvasSize() {
    const width = this.#canvas.width;
    const height = this.#canvas.height;

    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.#renderer.setSize(width, height);
    this.#camera.aspect = width / height;
    this.#camera.updateProjectionMatrix();
  }

  render(delta: number) {
    this.#updateCanvasSize();
    this.#mixers.forEach((mixer) => mixer.update(delta));
    this.#renderer.render(this.#scene, this.#camera);
    this.#stats.update();
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
    const { animationActions, animationMixer, scene } = await this.#parser.parse();

    // Calculate bounding box
    const boundingBox = new Box3().setFromObject(scene);
    const size = boundingBox.getSize(new Vector3());
    const center = boundingBox.getCenter(new Vector3());

    // Set camera position
    if (size.x === 0) size.setX(1);
    if (size.y === 0) size.setY(1);
    if (size.z === 0) size.setZ(1);
    this.#camera.position.set(size.x, size.y, size.z * 2);

    // Set camera rotation
    this.#camera.lookAt(center);
    this.#orbitControls.target.copy(center);

    // Set camera near and far
    const min = boundingBox.min.z === 0 ? 1 : boundingBox.min.z;
    const max = boundingBox.max.z === 0 ? 1 : boundingBox.max.z;
    this.#camera.near = Math.abs(min) / 1000;
    this.#camera.far = Math.abs(max) * 100;
    this.#camera.far = Math.max(this.#camera.far, 50);
    this.#camera.updateProjectionMatrix();

    // Add to scene
    this.#currentGltf = scene;
    this.#scene.add(scene);
    this.#mixers.set(scene.uuid, animationMixer);

    return {
      scene,
      animations: animationActions,
    };
  }

  export() {
    const exporter = new GLTFExporter();
    return exporter.exportAsBinary(this.#scene);
  }

  destroy() {
    // Dispose rendering context
    this.#renderer.dispose();

    // Dispose all objects
    disposeTree(this.#scene);

    // Clear scene
    this.#scene.clear();

    // Clear mixers
    this.#mixers.clear();

    // Remove stats
    document.body.removeChild(this.#stats.dom);
  }

  info() {
    return this.#renderer.info;
  }
}
