import { EventDispatcher } from "property-graph";

import { Engine } from "../Engine";
import { FakeWorker } from "../utils/FakeWorker";
import { PhysicsEvent } from "./events";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";

/**
 * Acts as an interface between the main thread and the physics thread.
 *
 * @group Modules
 */
export class PhysicsModule extends EventDispatcher<PhysicsEvent> {
  readonly engine: Engine;

  #worker: Worker | FakeWorker | null = null;

  ready = false;
  messageQueue: Array<{ message: ToPhysicsMessage; options?: StructuredSerializeOptions }> = [];

  constructor(engine: Engine) {
    super();

    this.engine = engine;

    this.#createWorker();
  }

  #createWorker() {
    this.ready = false;

    // Use a fake worker in development, for better HMR support
    if (process.env.NODE_ENV === "development") {
      import("./PhysicsThread").then(({ PhysicsThread }) => {
        this.#worker = new FakeWorker();

        const thread = new PhysicsThread(
          this.#worker.insidePort.postMessage.bind(this.#worker.insidePort)
        );

        this.#worker.insidePort.onmessage = thread.onmessage.bind(thread);
        this.#worker.outsidePort.onmessage = this.onmessage.bind(this);
      });
    } else {
      this.#worker = new Worker(new URL("./worker.ts", import.meta.url), {
        name: "physics",
        type: "module",
      });
      this.#worker.onmessage = this.onmessage;
    }

    this.send({
      data: {
        cameraYaw: this.engine.cameraYaw,
        input: this.engine.inputPosition,
        userPosition: this.engine.userPosition,
      },
      subject: "set_user_arrays",
    });

    this.send({ data: this.engine.controls, subject: "set_controls" });
    this.send({ data: this.engine.showColliders, subject: "toggle_collider_visuals" });
    if (this.engine.isPlaying) this.send({ data: this.engine.showColliders, subject: "start" });
  }

  onmessage = (event: MessageEvent<FromPhysicsMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;

        // Send queued messages
        this.messageQueue.forEach(({ message, options }) => {
          if (!this.#worker) throw new Error("Worker not found");
          this.#worker.postMessage(message, options);
        });

        this.messageQueue = [];
        break;
      }

      case "set_grounded": {
        this.engine.render.send({ data, subject: "set_grounded" });
        this.dispatchEvent({ data, type: "user_grounded" });
        break;
      }

      case "set_debug_buffers": {
        this.engine.render.send({ data, subject: "set_debug_buffers" });
        break;
      }
    }
  };

  /**
   * Sends a message to the physics worker.
   */
  send(message: ToPhysicsMessage, options?: StructuredSerializeOptions) {
    // If not ready, queue message
    if (!this.ready) {
      this.messageQueue.push({ message, options });
      return;
    }

    this.#worker?.postMessage(message, options);
  }

  /**
   * Destroy and re-create the physics thread.
   */
  async reset() {
    await this.destroy();
    this.#createWorker();
  }

  destroy() {
    this.ready = false;

    if (this.#worker instanceof FakeWorker) {
      this.#worker.postMessage({ data: null, subject: "destroy" });
    }

    this.#worker?.terminate();
  }
}
