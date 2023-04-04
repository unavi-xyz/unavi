import { EventDispatcher } from "property-graph";

import { Engine } from "../Engine";
import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderEvent } from "./events";
import { FromRenderMessage, RenderStats, ToRenderMessage } from "./messages";

/**
 * Acts as an interface between the main thread and the render thread.
 * Only runs the render thread in a web worker if `OffscreenCanvas` is supported.
 *
 * @group Modules
 */
export class RenderModule extends EventDispatcher<RenderEvent> {
  readonly engine: Engine;

  #worker: Worker | FakeWorker | null = null;

  ready = false;
  messageQueue: Array<{ message: ToRenderMessage; transferables?: Transferable[] }> = [];

  stats: RenderStats | null = null;

  constructor(engine: Engine) {
    super();

    this.engine = engine;

    // If OffscreenCanvas not supported, or in development, render on the main thread
    if (typeof OffscreenCanvas === "undefined" || process.env.NODE_ENV === "development") {
      import("./RenderThread").then(({ RenderThread }) => {
        this.#worker = new FakeWorker();

        const thread = new RenderThread(
          this.#worker.insidePort.postMessage.bind(this.#worker.insidePort),
          engine.canvas
        );

        this.#worker.insidePort.onmessage = thread.onmessage.bind(thread);
        this.#worker.outsidePort.onmessage = this.onmessage.bind(this);
      });
    } else {
      const offscreen = engine.canvas.transferControlToOffscreen();

      this.#worker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
        name: "render",
      });

      this.#worker.onmessage = this.onmessage.bind(this);

      // Send canvas to worker
      this.send({ subject: "set_canvas", data: offscreen }, [offscreen]);
    }

    this.send({
      subject: "set_size",
      data: { width: engine.canvas.width, height: engine.canvas.height },
    });

    this.send({ subject: "set_pixel_ratio", data: window.devicePixelRatio });

    this.send({
      subject: "set_user_arrays",
      data: {
        inputPosition: engine.inputPosition,
        inputRotation: engine.inputRotation,
        userPosition: engine.userPosition,
        userRotation: engine.userRotation,
        cameraPosition: engine.cameraPosition,
        cameraYaw: engine.cameraYaw,
      },
    });
  }

  onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;

        // Send queued messages
        this.messageQueue.forEach(({ message, transferables }) => {
          this.#worker?.postMessage(message, transferables);
        });

        this.messageQueue = [];
        break;
      }

      case "hovered_node":
      case "clicked_node": {
        this.dispatchEvent({ type: subject, data });
        break;
      }

      case "stats": {
        this.stats = data;
        break;
      }

      case "set_node_transform": {
        const node = this.engine.scene.node.store.get(data.nodeId);
        if (!node) throw new Error("Node not found");

        node.setTranslation(data.translation);
        node.setRotation(data.rotation);
        node.setScale(data.scale);
        break;
      }

      case "create_accessor": {
        this.engine.scene.accessor.create(data.json, data.id);
        break;
      }

      case "dispose_accessor": {
        const accessor = this.engine.scene.accessor.store.get(data);
        if (accessor) accessor.dispose();
        break;
      }

      case "create_primitive": {
        this.engine.scene.primitive.create(data.json, data.id);
        break;
      }

      case "dispose_primitive": {
        const primitive = this.engine.scene.primitive.store.get(data);
        if (primitive) primitive.dispose();
        break;
      }

      case "change_mesh": {
        const mesh = this.engine.scene.mesh.store.get(data.id);
        if (!mesh) throw new Error("Mesh not found");
        this.engine.scene.mesh.applyJSON(mesh, data.json);
        break;
      }
    }
  };

  /**
   * Sends a message to the render worker.
   * @param message The message to send.
   * @param transferables Transferable objects to send with the message.
   */
  send(message: ToRenderMessage, transferables?: Transferable[]) {
    // If not ready, queue message
    if (!this.ready) {
      this.messageQueue.push({ message, transferables });
      return;
    }

    this.#worker?.postMessage(message, transferables);
  }

  async destroy() {
    this.ready = false;

    if (this.#worker instanceof FakeWorker) {
      this.#worker.postMessage({ subject: "destroy", data: null });

      // Wait for the worker to process the message
      await new Promise<void>((resolve) => {
        setTimeout(() => {
          this.#worker?.terminate();
          resolve();
        }, 100);
      });
    } else {
      this.#worker?.terminate();
    }
  }
}
