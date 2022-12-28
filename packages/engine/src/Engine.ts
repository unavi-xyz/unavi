import { GLTFExporter } from "./exporter/GLTFExporter";
import { LoaderThread } from "./loader/LoaderThread";
import { InputManager } from "./main/InputManager";
import { MainScene } from "./main/MainScene";
import { PhysicsThread } from "./physics/PhysicsThread";
import { RenderThread } from "./render/RenderThread";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  skyboxPath?: string;
  avatarPath?: string;
  avatarAnimationsPath?: string;
}

/*
 * A multi-threaded 3D game engine.
 * Uses Web Workers to offload heavy tasks to separate threads.
 */
export class Engine {
  physicsThread: PhysicsThread;
  loaderThread: LoaderThread;
  renderThread: RenderThread;

  scene: MainScene;
  input: InputManager | null = null;

  running = false;

  constructor({
    canvas,
    camera = "player",
    enableTransformControls,
    preserveDrawingBuffer,
    skyboxPath,
    avatarPath,
    avatarAnimationsPath,
  }: EngineOptions) {
    // Create render thread
    this.renderThread = new RenderThread({
      canvas,
      engine: this,
      camera,
      enableTransformControls,
      preserveDrawingBuffer,
      skyboxPath,
      avatarPath,
      avatarAnimationsPath,
    });

    this.physicsThread = new PhysicsThread({ canvas, engine: this });
    this.loaderThread = new LoaderThread();
    this.scene = new MainScene({
      physicsThread: this.physicsThread,
      loaderThread: this.loaderThread,
      renderThread: this.renderThread,
    });
    this.input = new InputManager(this.renderThread, this.physicsThread, camera);

    // Once the threads are ready, create the player
    const createPlayer = async () => {
      await this.renderThread.waitForReady();
      await this.physicsThread.waitForReady();

      this.physicsThread.initPlayer();
    };

    if (camera === "player") createPlayer();
  }

  async waitForReady() {
    await this.physicsThread.waitForReady();
    await this.loaderThread.waitForReady();
    await this.renderThread.waitForReady();
  }

  async start() {
    await this.waitForReady();

    this.physicsThread.start();
    this.renderThread.start();

    this.running = true;
  }

  stop() {
    this.physicsThread.stop();
    this.renderThread.stop();

    this.running = false;
  }

  async export() {
    // Get scene
    const json = this.scene.toJSON(true);
    const data = await this.renderThread.export();

    // Export as glb
    const exporter = new GLTFExporter();
    const glb = await exporter.parse(json, data);

    return glb;
  }

  destroy() {
    this.physicsThread.destroy();
    this.loaderThread.destroy();
    this.renderThread.destroy();
    this.scene.destroy();
    this.input?.destroy();
  }
}
