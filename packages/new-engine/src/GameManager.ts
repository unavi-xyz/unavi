import { ColliderDesc, World } from "@dimforge/rapier3d";

import { RenderManager } from "./RenderManager";
import { FromGameMessage, IGLTF, ToGameLoadGltf } from "./types";

const gravity = { x: 0, y: -9.81, z: 0 };

export class GameManager {
  #worker = new Worker(new URL("./workers/Game.worker.js", import.meta.url), { type: "module" });
  #messageId = 0;

  #renderManager: RenderManager;

  #world = new World(gravity);

  constructor(renderManager: RenderManager) {
    this.#renderManager = renderManager;

    // Create ground
    const groundCollider = ColliderDesc.cuboid(10, 0.1, 10);
    this.#world.createCollider(groundCollider);
  }

  loadGltf(uri: string) {
    const message: ToGameLoadGltf = {
      id: this.#messageId++,
      type: "load_gltf",
      data: {
        uri,
      },
    };

    // Send message to worker
    this.#worker.postMessage(message);

    // Wait for worker to respond
    const promise = new Promise<IGLTF>((resolve, reject) => {
      const onMessage = async (event: MessageEvent<FromGameMessage>) => {
        const { type, data, id } = event.data;
        if (id !== message.id) return;
        this.#worker.removeEventListener("message", onMessage);

        if (type === "loaded_gltf") {
          const gltf = await this.#renderManager.createGLTF(data);
          resolve(gltf);
        } else {
          reject();
        }
      };

      this.#worker.addEventListener("message", onMessage);
    });

    return promise;
  }

  export() {
    return this.#renderManager.export();
  }

  update(delta: number) {
    this.#world.step();
  }

  destroy() {
    // Terminate worker
    this.#worker.terminate();
  }
}
