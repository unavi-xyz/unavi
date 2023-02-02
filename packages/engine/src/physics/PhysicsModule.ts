import { EventDispatcher } from "property-graph";

import { Engine } from "../Engine";
import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { PhysicsEvent } from "./events";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";

export class PhysicsModule extends EventDispatcher<PhysicsEvent> {
  readonly engine: Engine;

  #worker: Worker | FakeWorker | null = null;

  ready = false;
  messageQueue: Array<{ message: ToPhysicsMessage; transferables?: Transferable[] }> = [];

  constructor(engine: Engine) {
    super();

    this.engine = engine;

    // Use a fake worker in development, for better HMR support
    if (process.env.NODE_ENV === "production") {
      this.#worker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
        name: "physics",
      });
      this.#worker.onmessage = this.onmessage;
    } else {
      import("./PhysicsThread").then(({ PhysicsThread }) => {
        this.#worker = new FakeWorker();

        const thread = new PhysicsThread(
          this.#worker.insidePort.postMessage.bind(this.#worker.insidePort)
        );

        this.#worker.insidePort.onmessage = thread.onmessage.bind(thread);
        this.#worker.outsidePort.onmessage = this.onmessage.bind(this);
      });
    }

    this.send({
      subject: "set_user_arrays",
      data: {
        input: this.engine.inputPosition,
        userPosition: this.engine.userPosition,
        cameraYaw: this.engine.cameraYaw,
      },
    });
  }

  onmessage = (event: MessageEvent<FromPhysicsMessage>) => {
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

      case "set_grounded": {
        this.engine.render.send({ subject: "set_grounded", data });
        this.dispatchEvent({ type: "user_grounded", data });
        break;
      }
    }
  };

  send(message: ToPhysicsMessage, transferables?: Transferable[]) {
    // If not ready, queue message
    if (!this.ready) {
      this.messageQueue.push({ message, transferables });
      return;
    }

    this.#worker?.postMessage(message, transferables);
  }

  destroy() {
    this.send({ subject: "destroy", data: null });
    setTimeout(() => this.#worker?.terminate());
  }
}
