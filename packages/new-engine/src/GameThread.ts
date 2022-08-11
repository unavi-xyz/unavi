import { Engine } from "./Engine";
import { FromGameMessage, ToGameMessage } from "./types";

export class GameThread {
  #worker = new Worker(new URL("./workers/Game.worker.ts", import.meta.url), { type: "module" });

  constructor(engine: Engine) {
    this.#worker.onmessage = (event: MessageEvent<FromGameMessage>) => {
      const { subject, data } = event.data;

      switch (subject) {
        case "player_buffers":
          engine.renderManager.setPlayerBuffers(data);
          break;
      }
    };
  }

  initPlayer() {
    this.#postMessage({ subject: "init_player", data: null });
  }

  destroy() {
    this.#worker.terminate();
  }

  #postMessage = (message: ToGameMessage) => {
    this.#worker.postMessage(message);
  };
}
