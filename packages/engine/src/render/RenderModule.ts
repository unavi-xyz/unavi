import { EventDispatcher } from "property-graph";

import { Engine } from "../Engine";
import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderEvent } from "./events";
import { FromRenderMessage, ToRenderMessage } from "./messages";
import { RenderThread } from "./RenderThread";

export class RenderModule extends EventDispatcher<RenderEvent> {
  #worker: Worker | FakeWorker;

  #engine: Engine;

  ready = false;
  messageQueue: Array<{ message: ToRenderMessage; transferables?: Transferable[] }> = [];

  constructor(canvas: HTMLCanvasElement, engine: Engine) {
    super();

    this.#engine = engine;

    // If OffscreenCanvas is supported, render in a worker
    if (typeof OffscreenCanvas !== "undefined" && process.env.NODE_ENV === "production") {
      const offscreen = canvas.transferControlToOffscreen();

      this.#worker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
        name: "render",
      });

      this.#worker.onmessage = this.onmessage.bind(this);

      // Send canvas to worker
      this.toRenderThread({ subject: "set_canvas", data: offscreen }, [offscreen]);
    } else {
      // Otherwise render on the main thread, using a fake worker
      this.#worker = new FakeWorker();

      const thread = new RenderThread(
        this.#worker.insidePort.postMessage.bind(this.#worker.insidePort),
        canvas
      );

      this.#worker.insidePort.onmessage = thread.onmessage.bind(thread);
      this.#worker.outsidePort.onmessage = this.onmessage.bind(this);
    }

    this.toRenderThread({
      subject: "set_size",
      data: { width: canvas.width, height: canvas.height },
    });

    this.toRenderThread({ subject: "set_pixel_ratio", data: window.devicePixelRatio });
  }

  onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;

        // Send queued messages
        this.messageQueue.forEach(({ message, transferables }) => {
          this.#worker.postMessage(message, transferables);
        });

        this.messageQueue = [];
        break;
      }

      case "clicked_node": {
        this.dispatchEvent({ type: subject, data });
        break;
      }

      case "set_node_transform": {
        const node = this.#engine.modules.scene.node.store.get(data.nodeId);
        if (!node) throw new Error("Node not found");

        node.setTranslation(data.translation);
        node.setRotation(data.rotation);
        node.setScale(data.scale);
      }
    }
  };

  toRenderThread(message: ToRenderMessage, transferables?: Transferable[]) {
    // If not ready, queue message
    if (!this.ready) {
      this.messageQueue.push({ message, transferables });
      return;
    }

    this.#worker.postMessage(message, transferables);
  }

  destroy() {
    this.#worker.terminate();
  }
}
