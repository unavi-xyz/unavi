import { GameThread } from "./game/GameThread";
import { RenderThread } from "./render/RenderThread";

export interface EngineOptions {
  skyboxPath?: string;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
}

export class Engine {
  gameThread: GameThread;
  renderThread: RenderThread;

  #cameraType: "player" | "orbit";

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const {
      skyboxPath,
      camera = "player",
      enableTransformControls,
      preserveDrawingBuffer,
    } = { ...options };

    this.#cameraType = camera;

    this.renderThread = new RenderThread(canvas, {
      skyboxPath,
      camera,
      enableTransformControls,
      preserveDrawingBuffer,
    });

    this.gameThread = new GameThread(canvas, this.renderThread);
  }

  async start() {
    // Wait for workers to be ready
    await this.renderThread.waitForReady();
    await this.gameThread.waitForReady();

    // Init player
    if (this.#cameraType === "player") this.gameThread.initPlayer();

    // Start rendering
    this.renderThread.start();
  }

  stop() {
    this.renderThread.stop();
  }

  destroy() {
    this.gameThread.destroy();
    this.renderThread.destroy();
  }
}
