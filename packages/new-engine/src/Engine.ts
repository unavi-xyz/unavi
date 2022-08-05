import { GameManager } from "./GameManager";
import { RenderManager } from "./RenderManager";

const GAME_FPS = 60;
const GAME_INTERVAL = GAME_FPS;

export interface EngineOptions {
  canvas: HTMLCanvasElement;
  stats?: boolean;
  alpha?: boolean;
}

export class Engine {
  #animationFrameId: number | null = null;
  #lastGameTime = 0;
  #lastRenderTime = 0;

  #renderManager: RenderManager;
  #gameManager: GameManager;

  constructor({ canvas, stats, alpha }: EngineOptions) {
    this.#renderManager = new RenderManager({ canvas, stats, alpha });
    this.#gameManager = new GameManager(this.#renderManager);

    // Start render loop
    this.start();
  }

  #renderLoop(time: number) {
    const timeSeconds = time / 1000;

    // Request the next frame
    this.#animationFrameId = requestAnimationFrame(this.#renderLoop.bind(this));

    // Calculate delta
    const deltaGame = timeSeconds - this.#lastGameTime;
    const deltaRender = timeSeconds - this.#lastRenderTime;
    this.#lastRenderTime = timeSeconds;

    // Limit game updates to 60fps
    if (deltaGame > GAME_INTERVAL) {
      // Adjust for animation frame not being exactly 60fps
      this.#lastGameTime = timeSeconds - (this.#lastGameTime % GAME_INTERVAL);

      // Update game state
      this.#gameManager.update(deltaGame);
    }

    // Render
    this.#renderManager.render(deltaRender);
  }

  loadGltf(uri: string) {
    return this.#gameManager.loadGltf(uri);
  }

  export() {
    return this.#gameManager.export();
  }

  start() {
    this.stop();

    // Start render loop
    this.#lastGameTime = performance.now();
    this.#animationFrameId = requestAnimationFrame(this.#renderLoop.bind(this));
  }

  stop() {
    // Stop render loop
    if (this.#animationFrameId) {
      cancelAnimationFrame(this.#animationFrameId);
      this.#animationFrameId = null;
    }
  }

  info() {
    return this.#renderManager.info();
  }

  destroy() {
    // Destroy workers
    this.#gameManager.destroy();
    this.#renderManager.destroy();

    // Stop render loop
    this.stop();
  }

  get scene() {
    return this.#renderManager.scene;
  }

  get camera() {
    return this.#renderManager.camera;
  }

  get renderer() {
    return this.#renderManager.renderer;
  }
}
