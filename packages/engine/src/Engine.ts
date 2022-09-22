import { RenderThread } from "./render/RenderThread";
import { Scene } from "./scene/Scene";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  skyboxPath?: string;
}

export class Engine {
  scene = new Scene();
  renderThread: RenderThread;

  constructor({
    canvas,
    camera = "player",
    enableTransformControls,
    preserveDrawingBuffer,
    skyboxPath,
  }: EngineOptions) {
    this.renderThread = new RenderThread({
      canvas,
      camera,
      enableTransformControls,
      preserveDrawingBuffer,
      scene: this.scene,
      skyboxPath,
    });

    this.scene.addThread(
      this.renderThread.worker.postMessage.bind(this.renderThread.worker)
    );
  }

  async start() {
    await this.renderThread.waitForReady();
    this.renderThread.start();
  }

  stop() {}

  destroy() {
    this.scene.destroy();
    this.renderThread.destroy();
  }
}
