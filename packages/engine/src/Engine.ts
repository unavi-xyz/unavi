import { createWorld } from "bitecs";

import { LoaderThread } from "./LoaderThread";
import { RenderThread } from "./RenderThread";
import { config, deserialize } from "./ecs/components";

export interface EngineOptions {
  skyboxPath?: string;
  controls?: "orbit" | "player";
}

export class Engine {
  world = createWorld(config);

  renderThread: RenderThread;
  loaderThread = new LoaderThread();

  names: string[] = [];

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const { skyboxPath, controls = "player" } = { ...options };

    this.renderThread = new RenderThread(canvas, { skyboxPath, controls });

    this.start();
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
    this.loaderThread.destroy();
  }

  async loadGltf(path: string) {
    await this.loaderThread.waitForReady();
    const { data } = await this.loaderThread.loadGltf(path);
    this.names = data.assets.names;

    this.world = createWorld(config);
    deserialize(this.world, data.world);

    this.renderThread.loadScene(data.world, data.assets.images, data.assets.accessors);
  }
}
