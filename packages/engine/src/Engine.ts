import { GameThread } from "./game/GameThread";
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
 * Engine is a multi-threaded 3D game engine.
 * It uses a {@link GameThread} to run the game logic and a {@link RenderThread} to render the scene.
 * Scene state is stored in the {@link Scene} class.
 */
export class Engine {
  renderThread: RenderThread;
  gameThread: GameThread;
  scene: MainScene;

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
      engine: this,
      camera,
      enableTransformControls,
      preserveDrawingBuffer,
      skyboxPath,
    });

    // Create game thread
    this.gameThread = new GameThread({
      canvas,
      renderThread: this.renderThread,
    });

    this.gameThread.waitForReady().then(() => {
      if (camera === "player") this.gameThread.initPlayer();
    });

    this.scene = new MainScene({
      renderThread: this.renderThread,
      gameThread: this.gameThread,
    });
  }

  async waitForReady() {
    await this.renderThread.waitForReady();
    await this.gameThread.waitForReady();
  }

  async start() {
    await this.waitForReady();

    this.renderThread.start();
    this.gameThread.start();
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
