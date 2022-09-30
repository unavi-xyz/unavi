import { SceneJSON } from "../scene";
import { Transferable } from "../types";
import { FromLoaderMessage, ToLoaderMessage } from "./types";

/*
 * Acts as an interface between the main thread and the {@link LoaderWorker}.
 */
export class LoaderThread {
  ready = false;

  #worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
  });

  #onReady: Array<() => void> = [];

  constructor() {
    this.#worker.onmessage = this.#onmessage;
  }

  #onmessage = (event: MessageEvent<FromLoaderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready":
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      case "gltf_loaded":
        if (this.onGltfLoaded) this.onGltfLoaded(data);
        break;
    }
  };

  onGltfLoaded: ((data: { id: string; scene: SceneJSON }) => void) | null =
    null;

  postMessage(message: ToLoaderMessage, transfer?: Transferable[]) {
    this.#worker.postMessage(message, transfer);
  }

  waitForReady() {
    const promise = new Promise<void>((resolve) => {
      if (this.ready) resolve();
      this.#onReady.push(resolve);
    });
    return promise;
  }

  destroy() {
    this.#worker.terminate();
  }
}
