import { GameThread } from "./game/GameThread";
import { LoaderThread } from "./loader/LoaderThread";
import { MainScene } from "./main/MainScene";
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
  gameThread: GameThread;
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

    this.gameThread = new GameThread({
      canvas,
      renderThread: this.renderThread,
    });

    this.scene = new MainScene({
      gameThread: this.gameThread,
      loaderThread: this.loaderThread,
      renderThread: this.renderThread,
    });

    // Init the player once ready
    this.gameThread.waitForReady().then(() => {
      if (camera === "player") this.gameThread.initPlayer();
    });
  }

  async waitForReady() {
    await this.gameThread.waitForReady();
    await this.loaderThread.waitForReady();
    await this.renderThread.waitForReady();
  }

  async start() {
    await this.waitForReady();

    this.gameThread.start();
    this.renderThread.start();
  }

  stop() {
    this.gameThread.stop();
    this.renderThread.stop();
  }

  destroy() {
    this.gameThread.destroy();
    this.loaderThread.destroy();
    this.renderThread.destroy();
    this.scene.destroy();
  }
}
