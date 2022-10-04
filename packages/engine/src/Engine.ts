import { GLTFExporter } from "./exporter/GLTFExporter";
import { LoaderThread } from "./loader/LoaderThread";
import { MainScene } from "./main/MainScene";
import { PhysicsThread } from "./physics/PhysicsThread";
import { RenderThread } from "./render/RenderThread";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  skyboxPath?: string;
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

  constructor({
    canvas,
    camera = "player",
    enableTransformControls,
    preserveDrawingBuffer,
    skyboxPath,
  }: EngineOptions) {
    this.renderThread = new RenderThread({
      canvas,
      engine: this,
      camera,
      enableTransformControls,
      preserveDrawingBuffer,
      skyboxPath,
    });

    this.loaderThread = new LoaderThread();

    this.physicsThread = new PhysicsThread({
      canvas,
      renderThread: this.renderThread,
    });

    this.scene = new MainScene({
      physicsThread: this.physicsThread,
      loaderThread: this.loaderThread,
      renderThread: this.renderThread,
    });

    // Init the player once ready
    this.physicsThread.waitForReady().then(() => {
      if (camera === "player") this.physicsThread.initPlayer();
    });
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
  }

  stop() {
    this.physicsThread.stop();
    this.renderThread.stop();
  }

  async export() {
    const json = this.scene.toJSON(true);

    const exporter = new GLTFExporter();

    const glb = await exporter.parse(json);
    return glb;
  }

  destroy() {
    this.physicsThread.destroy();
    this.loaderThread.destroy();
    this.renderThread.destroy();
    this.scene.destroy();
  }
}
