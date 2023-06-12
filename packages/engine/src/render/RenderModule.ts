import { EventDispatcher } from "property-graph";

import { Engine } from "../Engine";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderEvent } from "./events";
import { FromRenderMessage, RenderStats, ToRenderMessage } from "./messages";
import type { RenderThread } from "./RenderThread";

/**
 * Acts as an interface between the main thread and the render thread.
 * Only runs the render thread in a web worker if `OffscreenCanvas` is supported.
 *
 * @group Modules
 */
export class RenderModule extends EventDispatcher<RenderEvent> {
  readonly engine: Engine;

  #worker: Worker | FakeWorker | null = null;
  renderThread: RenderThread | null = null;

  ready = false;
  messageQueue: Array<{ message: ToRenderMessage; options?: StructuredSerializeOptions }> = [];

  stats: RenderStats | null = null;

  constructor(engine: Engine) {
    super();

    this.engine = engine;

    // If using OffscreenCanvas, render in a web worker
    // Otherwise, render in the main thread
    if (engine.useOffscreenCanvas) {
      this.#worker = new Worker(new URL("./worker.ts", import.meta.url), {
        name: "render",
        type: "module",
      });

      this.#worker.onmessage = this.onmessage.bind(this);
    } else {
      import("./RenderThread").then(({ RenderThread }) => {
        this.#worker = new FakeWorker();

        const thread = new RenderThread(
          this.#worker.insidePort.postMessage.bind(this.#worker.insidePort)
        );
        thread.canvas = engine.canvas;

        this.#worker.insidePort.onmessage = thread.onmessage.bind(thread);
        this.#worker.outsidePort.onmessage = this.onmessage.bind(this);
      });
    }

    this.send({ data: window.devicePixelRatio, subject: "set_pixel_ratio" });

    this.send({
      data: {
        cameraPosition: engine.cameraPosition,
        cameraYaw: engine.cameraYaw,
        inputPosition: engine.inputPosition,
        inputRotation: engine.inputRotation,
        userPosition: engine.userPosition,
        userRotation: engine.userRotation,
      },
      subject: "set_user_arrays",
    });
  }

  onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;

        // Send queued messages
        this.messageQueue.forEach(({ message, options }) => {
          this.#worker?.postMessage(message, options);
        });

        this.messageQueue = [];
        break;
      }

      case "hovered_node":
      case "clicked_node": {
        this.dispatchEvent({ data, type: subject });
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
   */
  send(message: ToRenderMessage, options?: StructuredSerializeOptions) {
    // If not ready, queue message
    if (!this.ready) {
      this.messageQueue.push({ message, options });
      return;
    }

    this.#worker?.postMessage(message, options);
  }

  destroy() {
    this.ready = false;

    if (this.#worker instanceof FakeWorker) {
      this.#worker.postMessage({ data: null, subject: "destroy" });
    }

    this.#worker?.terminate();
  }
}
