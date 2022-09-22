import { GameThread } from "./game/GameThread";
import { RenderThread } from "./render/RenderThread";
import { Scene } from "./scene/Scene";

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  skyboxPath?: string;
}

/*
 * Engine is a multi-threaded 3D game engine.
 * It uses a {@link GameThread} to run the game logic and a {@link RenderThread} to render the scene.
 * Scene state is stored in the {@link Scene} class.
 */
export class Engine {
  scene = new Scene();
  renderThread: RenderThread;
  gameThread: GameThread;

  constructor({
    canvas,
    camera = "player",
    enableTransformControls,
    preserveDrawingBuffer,
    skyboxPath,
  }: EngineOptions) {
    // Create render thread
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

    // Create game thread
    this.gameThread = new GameThread({
      canvas,
      renderThread: this.renderThread,
    });

    this.gameThread.waitForReady().then(() => {
      if (camera === "player") this.gameThread.initPlayer();
      this.scene.addThread(
        this.gameThread.worker.postMessage.bind(this.gameThread.worker)
      );
    });
  }

  async start() {
    // Start render thread
    await this.renderThread.waitForReady();
    this.renderThread.start();

    // Start game thread
    this.gameThread.waitForReady().then(() => this.gameThread.start());
  }

  stop() {
    this.renderThread.stop();
    this.gameThread.stop();
  }

  destroy() {
    this.scene.destroy();
    this.renderThread.destroy();
    this.gameThread.destroy();
  }
}
