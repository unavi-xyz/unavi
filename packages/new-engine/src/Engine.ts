import { RenderThread } from "./RenderThread";
import { TreeItem } from "./classes/TreeItem";

export interface EngineOptions {
  skyboxPath?: string;
}

const defaultOptions = {
  skyboxPath: undefined,
};

export class Engine {
  renderThread: RenderThread;
  tree = new TreeItem();

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const { skyboxPath } = Object.assign(defaultOptions, options);

    this.tree.threeUUID = "root";
    this.renderThread = new RenderThread(canvas);

    if (skyboxPath) {
      this.renderThread.createSkybox(skyboxPath);
    }
  }

  destroy() {
    this.renderThread.destroy();
  }
}
