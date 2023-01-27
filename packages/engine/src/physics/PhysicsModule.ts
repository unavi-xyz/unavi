import { Engine } from "../Engine";
import { Transferable } from "../types";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";

export class PhysicsModule {
  readonly engine: Engine;
  readonly #worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
    name: "physics",
  });

  ready = false;
  messageQueue: Array<{ message: ToPhysicsMessage; transferables?: Transferable[] }> = [];

  positionArray: Int32Array | null = null;
  rotationArray: Int32Array | null = null;

  constructor(engine: Engine) {
    this.engine = engine;
    this.#worker.onmessage = this.onmessage;
  }

  onmessage = (event: MessageEvent<FromPhysicsMessage>) => {
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

      case "set_player_arrays": {
        this.positionArray = data.position;
        this.rotationArray = data.rotation;

        this.engine.input.inputArray = data.input;
        this.engine.input.rotationArray = data.rotation;

        this.engine.render.send({
          subject: "set_player_arrays",
          data: { position: data.position, rotation: data.rotation },
        });
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

    this.#worker.postMessage(message, transferables);
  }

  destroy() {
    this.#worker.terminate();
  }
}
