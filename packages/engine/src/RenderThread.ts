import { TypedArray } from "bitecs";

import { RenderWorker } from "./render/RenderWorker";
import { FromRenderMessage, RenderWorkerOptions, ToRenderMessage } from "./render/types";
import { Transferable } from "./types";
import { FakeWorker } from "./utils/FakeWorker";

export class RenderThread {
  ready = false;

  #worker: Worker | FakeWorker;
  #onReady: Array<() => void> = [];

  constructor(canvas: HTMLCanvasElement, options: RenderWorkerOptions) {
    // Create worker if browser supports OffscreenCanvas
    // if (typeof OffscreenCanvas !== "undefined") {
    //   const offscreen = canvas.transferControlToOffscreen();
    //   this.#worker = new Worker(new URL("./workers/Render.worker.ts", import.meta.url), {
    //     type: "module",
    //   });
    //   // Initialize worker
    //   this.postMessage({ subject: "init", data: { canvas: offscreen, options } }, [offscreen]);
    //   return
    // }

    // Otherwise, create a fake worker that runs in the main thread
    this.#worker = new FakeWorker();
    const renderWorker = new RenderWorker(
      this.#worker.workerPort.postMessage.bind(this.#worker.workerPort),
      canvas
    );
    this.#worker.workerPort.onmessage = renderWorker.onmessage;

    // On message from worker
    this.#worker.onmessage = this.#onmessage.bind(this);

    // Initialize worker
    this.#postMessage({ subject: "init", data: options });
  }

  waitForReady() {
    const promise = new Promise<void>((resolve) => {
      if (this.ready) resolve();
      this.#onReady.push(resolve);
    });
    return promise;
  }

  loadScene(worldBuffer: ArrayBuffer, images: ImageBitmap[], accessors: TypedArray[]) {
    this.#postMessage({ subject: "load_scene", data: { worldBuffer, images, accessors } });
  }

  start() {
    this.#postMessage({ subject: "start", data: null });
  }

  stop() {
    this.#postMessage({ subject: "stop", data: null });
  }

  destroy() {
    this.#worker.postMessage({ subject: "destroy", data: null });
    setTimeout(() => this.#worker.terminate());
  }

  #onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;
    switch (subject) {
      case "ready":
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      default:
        throw new Error(`Unknown message subject: ${subject}`);
    }
  };

  #postMessage(message: ToRenderMessage, transfer?: Transferable[]) {
    this.#worker.postMessage(message, transfer);
  }
}
