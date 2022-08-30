import { Changed, createWorld, defineSerializer, resetWorld } from "bitecs";

import { LoaderThread } from "./LoaderThread";
import { RenderThread } from "./RenderThread";
import { config, deserialize } from "./ecs/components";

export interface EngineOptions {
  skyboxPath?: string;
  camera?: "orbit" | "player";
  enableTransformControls?: boolean;
}

export class Engine {
  world = createWorld(config);

  renderThread: RenderThread;
  loaderThread = new LoaderThread();

  #serializeChanged = defineSerializer(config.map((c) => Changed(c)));

  constructor(canvas: HTMLCanvasElement, options?: EngineOptions) {
    const { skyboxPath, camera = "player", enableTransformControls } = { ...options };

    this.renderThread = new RenderThread(canvas, { skyboxPath, camera, enableTransformControls });

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
    resetWorld(this.world);
  }

  updateScene() {
    const buffer = this.#serializeChanged(this.world);
    this.renderThread.updateScene(buffer);
  }

  async loadGltf(path: string) {
    await this.loaderThread.waitForReady();
    const { data } = await this.loaderThread.loadGltf(path);

    this.world = createWorld(config);
    deserialize(this.world, data.world);

    this.renderThread.loadScene(data.world, data.assets.images, data.assets.accessors);
  }
}
