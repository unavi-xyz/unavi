import { SceneJSON } from "../scene";
import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { LoaderWorker } from "./LoaderWorker";
import { FromLoaderMessage, ToLoaderMessage } from "./types";

/*
 * Acts as an interface between the main thread and the {@link LoaderWorker}.
 */
export class LoaderThread {
  ready = false;

  #worker: FakeWorker;

  #onReady: Array<() => void> = [];

  constructor() {
    this.#worker = new FakeWorker();

    const loaderWorker = new LoaderWorker(
      this.#worker.workerPort.postMessage.bind(this.#worker.workerPort)
    );

    this.#worker.workerPort.onmessage = loaderWorker.onmessage.bind(loaderWorker);

    this.#worker.onmessage = this.#onmessage;
  }

  #onmessage = (event: MessageEvent<FromLoaderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      }

      case "gltf_loaded": {
        if (this.onGltfLoaded) this.onGltfLoaded(data);
        break;
      }
    }
  };

  onGltfLoaded: ((data: { id: string; scene: SceneJSON }) => void) | null = null;

  postMessage(message: ToLoaderMessage, transfer?: Transferable[]) {
    this.#worker.postMessage(message, transfer);
  }

  waitForReady() {
    return new Promise<void>((resolve) => {
      if (this.ready) {
        resolve();
        return;
      }

      this.#onReady.push(resolve);
    });
  }

  destroy() {
    this.#worker.terminate();
  }
}
