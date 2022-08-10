import { RenderManager } from "./RenderManager";

export interface EngineOptions {
  skyboxPath?: string;
}

const defaultOptions = {
  skyboxPath: undefined,
};

export class Engine {
  renderManager: RenderManager;

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const { skyboxPath } = Object.assign(defaultOptions, options);

    this.renderManager = new RenderManager(canvas, { skyboxPath });
  }

  destroy() {
    this.renderManager.destroy();
  }
}
