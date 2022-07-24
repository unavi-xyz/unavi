import { AnimationAction, Group } from "three";

import { RenderManager } from "./RenderManager";
import { FromGameMessage, ToGameWorkerLoadGltf } from "./types";

export interface IGLTF {
  scene: Group;
  animations: AnimationAction[];
}

export class GameManager {
  private _worker = new Worker(new URL("./workers/Game.worker.ts", import.meta.url));
  private _messageId = 0;

  private _renderManager: RenderManager;

  constructor(renderManager: RenderManager) {
    this._renderManager = renderManager;
  }

  public loadGltf(uri: string) {
    const message: ToGameWorkerLoadGltf = {
      id: this._messageId++,
      type: "load_gltf",
      data: {
        uri,
      },
    };

    // Send message to worker
    this._worker.postMessage(message);

    // Wait for worker to respond
    const promise = new Promise<IGLTF>((resolve, reject) => {
      const onMessage = async (event: MessageEvent<FromGameMessage>) => {
        const { type, data, id } = event.data;
        if (id !== message.id) return;
        this._worker.removeEventListener("message", onMessage);

        if (type === "loaded_gltf") {
          const gltf = await this._renderManager.createGLTF(data);
          resolve(gltf);
        } else {
          reject();
        }
      };

      this._worker.addEventListener("message", onMessage);
    });

    return promise;
  }

  public update(delta: number) {}

  public destroy() {
    //terminate worker
    this._worker.terminate();
  }
}
