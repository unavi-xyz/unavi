import { GameManager } from "./GameManager";
import { RenderManager } from "./RenderManager";

const GAME_FPS = 60;
const GAME_INTERVAL = GAME_FPS;

export interface EngineOptions {
  canvas: HTMLCanvasElement;
}

export class Engine {
  private _animationFrameId: number | null = null;
  private _lastGameTime = 0;
  private _lastRenderTime = 0;

  private _renderManager: RenderManager;
  private _gameManager: GameManager;

  constructor({ canvas }: EngineOptions) {
    this._renderManager = new RenderManager(canvas);
    this._gameManager = new GameManager(this._renderManager);

    // Start render loop
    this.start();
  }

  private _renderLoop(time: number) {
    const timeSeconds = time / 1000;

    // Request the next frame
    this._animationFrameId = requestAnimationFrame(this._renderLoop.bind(this));

    // Calculate delta
    const deltaGame = timeSeconds - this._lastGameTime;
    const deltaRender = timeSeconds - this._lastRenderTime;
    this._lastRenderTime = timeSeconds;

    // Limit game updates to 60fps
    if (deltaGame > GAME_INTERVAL) {
      // Adjust for animation frame not being exactly 60fps
      this._lastGameTime = timeSeconds - (this._lastGameTime % GAME_INTERVAL);

      // Update game state
      this._gameManager.update(deltaGame);
    }

    // Render
    this._renderManager.render(deltaRender);
  }

  public loadGltf(uri: string) {
    return this._gameManager.loadGltf(uri);
  }

  public start() {
    // Start render loop
    this._lastGameTime = performance.now();
    this._animationFrameId = requestAnimationFrame(this._renderLoop.bind(this));
  }

  public stop() {
    // Stop render loop
    if (this._animationFrameId) {
      cancelAnimationFrame(this._animationFrameId);
      this._animationFrameId = null;
    }
  }

  public destroy() {
    // Destroy workers
    this._gameManager.destroy();
    this._renderManager.destroy();

    // Stop render loop
    this.stop();
  }
}
