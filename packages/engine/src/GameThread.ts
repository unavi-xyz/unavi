import { Engine } from "./Engine";
import { FromGameMessage, ToGameMessage } from "./types";

export class GameThread {
  #worker = new Worker(new URL("./workers/Game.worker", import.meta.url), { type: "module" });

  ready = false;
  #readyListeners: Array<() => void> = [];

  constructor(engine: Engine) {
    this.#worker.onmessage = (event: MessageEvent<FromGameMessage>) => {
      const { subject, data } = event.data;

      switch (subject) {
        case "ready":
          this.ready = true;
          this.#readyListeners.forEach((listener) => listener());
          break;
        case "player_buffers":
          engine.renderManager.setPlayerBuffers(data);
          break;
      }
    };
  }

  waitForReady() {
    const promise: Promise<void> = new Promise((resolve) => {
      this.#readyListeners.push(resolve);
    });
    return promise;
  }

  initPlayer() {
    this.#postMessage({ subject: "init_player", data: null });
  }

  jump() {
    this.#postMessage({ subject: "jumping", data: true });
  }

  destroy() {
    this.#worker.terminate();
  }

  #postMessage = (message: ToGameMessage) => {
    this.#worker.postMessage(message);
  };
}
