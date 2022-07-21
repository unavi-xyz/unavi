import {
  AmbientLight,
  AnimationMixer,
  Box3,
  DirectionalLight,
  Mesh,
  Object3D,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  Vector3,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import Stats from "three/examples/jsm/libs/stats.module";

import { LoadedGLTF } from "./gltf";
import { GLTFParser } from "./gltf/GLTFParser";

export class RenderWorker {
  private _scene = new Scene();
  private _canvasWidth = 0;
  private _canvasHeight = 0;
  private _mixers = new Map<string, AnimationMixer>();
  private _parser: GLTFParser | null = null;

  private _orbitControls: OrbitControls;
  private _camera: PerspectiveCamera;
  private _renderer: WebGLRenderer;
  private _canvas: HTMLCanvasElement;
  private _stats: Stats;

  constructor(canvas: HTMLCanvasElement) {
    this._canvas = canvas;
    this._renderer = new WebGLRenderer({ canvas, antialias: true, alpha: true });
    this._renderer.setPixelRatio(window.devicePixelRatio);
    this._renderer.setSize(canvas.width, canvas.height);
    this._renderer.outputEncoding = sRGBEncoding;
    this._renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this._renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this._scene.environment = environment;

    // Lights
    const ambientLight = new AmbientLight(0xffffff, 0.5);
    this._scene.add(ambientLight);

    const directionalLight = new DirectionalLight(0xffffff, 0.5);
    directionalLight.position.set(0, 0, 1);
    this._scene.add(directionalLight);

    // Camera
    this._camera = new PerspectiveCamera(75, canvas.width / canvas.height);

    // Stats
    this._stats = Stats();
    this._stats.setMode(2);
    this._stats.dom.id = "stats";
    document.body.appendChild(this._stats.dom);

    // Controls
    this._orbitControls = new OrbitControls(this._camera, canvas);
  }

  private _updateCanvasSize() {
    const width = this._canvas.width;
    const height = this._canvas.height;

    if (width === this._canvasWidth && height === this._canvasHeight) return;

    this._canvasWidth = width;
    this._canvasHeight = height;

    this._renderer.setSize(width, height);
    this._camera.aspect = width / height;
    this._camera.updateProjectionMatrix();
  }

  public render(delta: number) {
    this._updateCanvasSize();
    this._mixers.forEach((mixer) => mixer.update(delta));
    this._renderer.render(this._scene, this._camera);
    this._stats.update();
  }

  public async createGLTF(data: LoadedGLTF) {
    // Parse gltf
    this._parser = new GLTFParser(data);
    const { animationMixer, group } = await this._parser.parse();

    // Calculate bounding box
    const boundingBox = new Box3().setFromObject(group);
    const size = boundingBox.getSize(new Vector3());
    const center = boundingBox.getCenter(new Vector3());

    // Set camera position
    if (size.z === 0) size.setZ(1);
    this._camera.position.set(size.x, size.y, size.z * 2);

    // Set camera rotation
    this._camera.lookAt(center);
    this._orbitControls.target.copy(center);

    // Set camera near and far
    const min = boundingBox.min.z === 0 ? 1 : boundingBox.min.z;
    const max = boundingBox.max.z === 0 ? 1 : boundingBox.max.z;
    this._camera.near = Math.abs(min) / 100;
    this._camera.far = Math.abs(max) * 100;
    this._camera.updateProjectionMatrix();

    // Add to scene
    this._scene.add(group);
    this._mixers.set(group.uuid, animationMixer);
  }

  public destroy() {
    // Dispose rendering context
    this._renderer.dispose();

    // Dispose scene
    this._scene.traverse((object) => {
      if (object instanceof Mesh) {
        object.geometry.dispose();
        object.material.dispose();
      }
    });

    // Clear mixers
    this._mixers.clear();

    // Remove stats
    document.body.removeChild(this._stats.dom);
  }
}
