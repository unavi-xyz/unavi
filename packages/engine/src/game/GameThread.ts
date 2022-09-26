import { RenderThread } from "../render/RenderThread";
import { Transferable } from "../types";
import { Player } from "./Player";
import { FromGameMessage, ToGameMessage } from "./types";

export interface GameThreadOptions {
  canvas: HTMLCanvasElement;
  renderThread: RenderThread;
}

/*
 * Acts as an interface between the main thread and the {@link GameWorker}.
 */
export class GameThread {
  #worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
  });

  ready = false;
  #readyListeners: Array<() => void> = [];

  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;

  #player: Player | null = null;

  constructor({ canvas, renderThread }: GameThreadOptions) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;

    this.#worker.onmessage = this.#onmessage;
  }

  #onmessage = (event: MessageEvent<FromGameMessage>) => {
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

  postMessage = (message: ToGameMessage, transfer?: Transferable[]) => {
    this.#worker.postMessage(message, transfer);
  };

  start() {
    this.postMessage({ subject: "start", data: null });
  }

  stop() {
    this.postMessage({ subject: "stop", data: null });
  }

  waitForReady() {
    const promise: Promise<void> = new Promise((resolve) => {
      this.#readyListeners.push(resolve);
    });
    return promise;
  }

  initPlayer() {
    this.#player = new Player(this.#canvas, this.#renderThread, this);
    this.postMessage({ subject: "init_player", data: null });
  }

  jump() {
    this.postMessage({ subject: "jumping", data: true });
  }

  destroy() {
    this.#worker.terminate();
    if (this.#player) this.#player.destroy();
  }
}
