import { FromLoaderLoadedGLTF, FromLoaderMessage, ToLoaderMessage } from "./types";

export class LoaderThread {
  ready = false;

  #worker = new Worker(new URL("./workers/Loader.worker.ts", import.meta.url), {
    type: "module",
  });

  #onReady: Array<() => void> = [];
  #messageId = 0;
  #gltfLoadListeners = new Map<number, (data: FromLoaderLoadedGLTF) => void>();

  constructor() {
    this.#worker.onmessage = this.#onmessage.bind(this);
  }

  waitForReady() {
    const promise = new Promise<void>((resolve) => {
      if (this.ready) resolve();
      this.#onReady.push(resolve);
    });
    return promise;
  }

  loadGltf(path: string) {
    const id = this.#messageId++;

    const promise = new Promise<FromLoaderLoadedGLTF>((resolve) => {
      this.#gltfLoadListeners.set(id, resolve);
    });

    this.#postMessage({ subject: "load_gltf", data: path, id });

    return promise;
  }

  destroy() {
    this.#worker.terminate();
  }

  #postMessage(message: ToLoaderMessage, transfer?: Transferable[]) {
    this.#worker.postMessage(message, transfer);
  }

  #onmessage(event: MessageEvent<FromLoaderMessage>) {
    const { subject, id } = event.data;

    switch (subject) {
      case "ready":
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      case "gltf_loaded":
        if (id !== undefined && this.#gltfLoadListeners.has(id)) {
          const listener = this.#gltfLoadListeners.get(id);
          this.#gltfLoadListeners.delete(id);
          if (listener) listener(event.data);
        }
    }
  }
}
