import { InputModule } from "../input/InputModule";
import { RenderModule } from "../render/RenderModule";
import { Transferable } from "../types";
import { FromPhysicsMessage, ToPhysicsMessage } from "./messages";

export class PhysicsModule {
  #worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
    name: "physics",
  });

  #input: InputModule;
  #render: RenderModule;

  ready = false;
  messageQueue: Array<{ message: ToPhysicsMessage; transferables?: Transferable[] }> = [];

  constructor(input: InputModule, render: RenderModule) {
    this.#input = input;
    this.#render = render;

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
        this.#input.inputArray = data.input;
        this.#input.rotationArray = data.rotation;

        this.#render.toRenderThread({
          subject: "set_player_arrays",
          data: { position: data.position, rotation: data.rotation },
        });
        break;
      }
    }
  };

  toPhysicsThread(message: ToPhysicsMessage, transferables?: Transferable[]) {
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
