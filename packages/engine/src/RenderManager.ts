import {
  CubeTextureLoader,
  FogExp2,
  MathUtils,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  Vector2,
  Vector3,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { disposeObject } from "./utils/disposeObject";

const DAMPEN_FACTOR = 2;
const SPEED = 3;

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

  #playerInputVector = new Vector2();
  #playerPosition: Float32Array | null = null;
  #playerVelocity: Float32Array | null = null;
  #inputMomentum = new Vector2();
  #inputYChangeTime = 0;
  #inputXChangeTime = 0;

  #tempVec2 = new Vector2();
  #tempVec3 = new Vector3();

  constructor(canvas: HTMLCanvasElement, options?: RenderManagerOptions) {
    const { skyboxPath } = { ...defaultOptions, ...options };

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

    // Fog
    this.scene.fog = new FogExp2(0xeefaff, 0.005);

    // Skybox
    if (skyboxPath) {
      this.scene.background = new CubeTextureLoader()
        .setPath(skyboxPath)
        .load(["right.bmp", "left.bmp", "top.bmp", "bottom.bmp", "front.bmp", "back.bmp"]);
    }

    // Camera
    this.camera = new PerspectiveCamera(75, canvas.width / canvas.height, 0.1, 1000);
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
    disposeObject(this.scene);
  }

  setPlayerInputVector(input: Vector2) {
    if (Math.sign(input.x) !== Math.sign(this.#playerInputVector.x)) {
      this.#inputXChangeTime = Date.now();
    }

    if (Math.sign(input.y) !== Math.sign(this.#playerInputVector.y)) {
      this.#inputYChangeTime = Date.now();
    }

    this.#playerInputVector.copy(input);
  }

  setPlayerBuffers({ position, velocity }: { position: Float32Array; velocity: Float32Array }) {
    this.#playerPosition = position;
    this.#playerVelocity = velocity;
  }

  #render() {
    this.#animationFrameId = requestAnimationFrame(() => this.#render());
    if (!this.renderer || !this.camera) return;

    const deltaX = Date.now() - this.#inputXChangeTime;
    const deltaY = Date.now() - this.#inputYChangeTime;

    // Dampen input
    this.#inputMomentum.x = MathUtils.damp(
      this.#inputMomentum.x,
      this.#playerInputVector.x,
      DAMPEN_FACTOR,
      deltaX / 1000
    );

    this.#inputMomentum.y = MathUtils.damp(
      this.#inputMomentum.y,
      this.#playerInputVector.y,
      DAMPEN_FACTOR,
      deltaY / 1000
    );

    if (Math.abs(this.#inputMomentum.x) < 0.001) this.#inputMomentum.x = 0;
    if (Math.abs(this.#inputMomentum.y) < 0.001) this.#inputMomentum.y = 0;

    // Rotate input vector by camera direction
    const direction = this.camera.getWorldDirection(this.#tempVec3);
    const angle = Math.atan2(direction.x, direction.z);
    const velocity = this.#tempVec2
      .set(this.#inputMomentum.x, this.#inputMomentum.y)
      .rotateAround(new Vector2(0, 0), -angle)
      .multiplyScalar(SPEED);

    // Send velocity to game thread
    if (this.#playerVelocity) {
      this.#playerVelocity[0] = velocity.x;
      this.#playerVelocity[1] = velocity.y;
    }

    // Apply player position
    if (this.#playerPosition) {
      this.camera.position.set(
        this.#playerPosition[0],
        this.#playerPosition[1],
        this.#playerPosition[2]
      );
    }

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
