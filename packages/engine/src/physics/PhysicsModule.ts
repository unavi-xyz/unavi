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
        type: "module",
        name: "physics",
      });
      this.#worker.onmessage = this.onmessage;
    }

    this.send({
      subject: "set_user_arrays",
      data: {
        input: this.engine.inputPosition,
        userPosition: this.engine.userPosition,
        cameraYaw: this.engine.cameraYaw,
      },
    });

    this.send({ subject: "set_controls", data: this.engine.controls });
    this.send({ subject: "toggle_collider_visuals", data: this.engine.showColliders });
    if (this.engine.isPlaying) this.send({ subject: "start", data: this.engine.showColliders });
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
        this.engine.render.send({ subject: "set_grounded", data });
        this.dispatchEvent({ type: "user_grounded", data });
        break;
      }

      case "set_debug_buffers": {
        this.engine.render.send({ subject: "set_debug_buffers", data });
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
      this.#worker.postMessage({ subject: "destroy", data: null });
    }

    this.#worker?.terminate();
  }
}
