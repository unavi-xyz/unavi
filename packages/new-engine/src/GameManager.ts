import { RenderManager } from "./RenderManager";
import { FromGameMessage, IGLTF, ToGameWorkerLoadGltf } from "./types";

export class GameManager {
  #worker = new Worker(new URL("./workers/Game.worker.ts", import.meta.url));
  #messageId = 0;

  #renderManager: RenderManager;

  constructor(renderManager: RenderManager) {
    this.#renderManager = renderManager;
  }

  loadGltf(uri: string) {
    const message: ToGameWorkerLoadGltf = {
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

  update(delta: number) {}

  destroy() {
    //terminate worker
    this.#worker.terminate();
  }
}
