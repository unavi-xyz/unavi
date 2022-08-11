import { GameThread } from "./GameThread";
import { RenderManager } from "./RenderManager";
import { Player } from "./player/Player";

export interface EngineOptions {
  skyboxPath?: string;
  player?: boolean;
}

const defaultOptions = {
  skyboxPath: undefined,
  player: false,
};

export class Engine {
  gameThread = new GameThread(this);
  renderManager: RenderManager;

  #player: Player | null = null;

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const { skyboxPath, player } = Object.assign(defaultOptions, options);

    this.renderManager = new RenderManager(canvas, { skyboxPath });

    if (player) this.#player = new Player(this);
  }

  destroy() {
    this.renderManager.destroy();
    this.gameThread.destroy();
    if (this.#player) this.#player.destroy();
  }
}
