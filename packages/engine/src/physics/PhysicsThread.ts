import { RenderThread } from "../render/RenderThread";
import { Transferable } from "../types";
import { Player } from "./Player";
import { FromPhysicsMessage, ToPhysicsMessage } from "./types";

export interface PhysicsThreadOptions {
  canvas: HTMLCanvasElement;
  renderThread: RenderThread;
}

/*
 * Acts as an interface between the main thread and the {@link PhysicsWorker}.
 */
export class PhysicsThread {
  #worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
  });

  ready = false;
  #readyListeners: Array<() => void> = [];

  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;

  #player: Player | null = null;

  constructor({ canvas, renderThread }: PhysicsThreadOptions) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;

    this.#worker.onmessage = this.#onmessage;
  }

  #onmessage = (event: MessageEvent<FromPhysicsMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready":
        this.ready = true;
        this.#readyListeners.forEach((listener) => listener());
        break;
      case "player_buffers":
        this.#renderThread.setPlayerBuffers(data);
        break;
    }
  };

  postMessage = (message: ToPhysicsMessage, transfer?: Transferable[]) => {
    this.#worker.postMessage(message, transfer);
  };

  start() {
    this.postMessage({ subject: "start", data: null });
  }

  stop() {
    this.postMessage({ subject: "stop", data: null });
  }

  waitForReady() {
    return new Promise<void>((resolve) => {
      if (this.ready) {
        resolve();
        return;
      }

      this.#readyListeners.push(resolve);
    });
  }

  initPlayer() {
    this.#player = new Player(this.#canvas, this.#renderThread, this);
    this.postMessage({ subject: "init_player", data: null });
  }

  jump() {
    this.postMessage({ subject: "jumping", data: true });
  }

  destroy() {
    this.stop();
    setTimeout(() => this.#worker.terminate());

    this.#player?.destroy();
  }
}
