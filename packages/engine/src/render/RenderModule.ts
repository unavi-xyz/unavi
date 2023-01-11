import { EventDispatcher } from "property-graph";

import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderEvent } from "./events";
import { FromRenderMessage, ToRenderMessage } from "./messages";
import { RenderThread } from "./RenderThread";

export class RenderModule extends EventDispatcher<RenderEvent> {
  #renderWorker: Worker | FakeWorker;

  constructor(canvas: HTMLCanvasElement) {
    super();

    // If OffscreenCanvas is supported, render in a worker
    if (typeof OffscreenCanvas !== "undefined") {
      const offscreen = canvas.transferControlToOffscreen();

      this.#renderWorker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
        name: "render",
      });

      this.#renderWorker.onmessage = this.onmessage.bind(this);

      // Send canvas to worker
      this.toRenderThread({ subject: "set_canvas", data: offscreen }, [offscreen]);
    } else {
      // Otherwise render on the main thread, using a fake worker
      this.#renderWorker = new FakeWorker();

      const thread = new RenderThread(
        this.#renderWorker.insidePort.postMessage.bind(this.#renderWorker.insidePort)
      );

      this.#renderWorker.insidePort.onmessage = thread.onmessage.bind(thread);
      this.#renderWorker.outsidePort.onmessage = this.onmessage.bind(this);

      // Send canvas to worker
      this.toRenderThread({ subject: "set_canvas", data: canvas });
    }

    this.toRenderThread({
      subject: "set_size",
      data: { width: canvas.width, height: canvas.height },
    });

    this.toRenderThread({ subject: "set_pixel_ratio", data: window.devicePixelRatio });
  }

  toRenderThread(message: ToRenderMessage, transferables?: Transferable[]) {
    this.#renderWorker.postMessage(message, transferables);
  }

  onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "clicked_node": {
        this.dispatchEvent({ type: subject, data });
        break;
      }
    }
  };
}
