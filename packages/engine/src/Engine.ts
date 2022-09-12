import { RenderThread } from "./render/RenderThread";

export interface EngineOptions {
  skyboxPath?: string;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
}

export class Engine {
  renderThread: RenderThread;
  // loaderThread = new LoaderThread();

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const {
      skyboxPath,
      camera = "player",
      enableTransformControls,
      preserveDrawingBuffer,
    } = { ...options };

    this.renderThread = new RenderThread(canvas, {
      skyboxPath,
      camera,
      enableTransformControls,
      preserveDrawingBuffer,
    });
  }

  async start() {
    await this.renderThread.waitForReady();
    this.renderThread.start();
  }

  stop() {
    this.renderThread.stop();
  }

  destroy() {
    this.renderThread.destroy();
    // this.loaderThread.destroy();
  }
}
