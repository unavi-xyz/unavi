import { RenderThread } from "../render/RenderThread";
import { Entity } from "../types";
import { Player } from "./Player";
import { FromGameMessage, ToGameMessage } from "./types";

export class GameThread {
  #worker = new Worker(new URL("../workers/Game.worker.ts", import.meta.url), {
    type: "module",
  });

  ready = false;
  #readyListeners: Array<() => void> = [];

  #canvas: HTMLCanvasElement;
  #renderThread: RenderThread;

  #player: Player | null = null;

  constructor(canvas: HTMLCanvasElement, renderThread: RenderThread) {
    this.#canvas = canvas;
    this.#renderThread = renderThread;

    this.#worker.onmessage = (event: MessageEvent<FromGameMessage>) => {
      const { subject, data } = event.data;

      switch (subject) {
        case "ready":
          this.ready = true;
          this.#readyListeners.forEach((listener) => listener());
          break;
        case "player_buffers":
          renderThread.setPlayerBuffers(data);
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
    this.#player = new Player(this.#canvas, this.#renderThread, this);
    this.#postMessage({ subject: "init_player", data: null });
  }

  jump() {
    this.#postMessage({ subject: "jumping", data: true });
  }

  setPhysics(entity: Entity) {
    this.#postMessage({
      subject: "set_physics",
      data: {
        entityId: entity.id,
        collider: entity.collider,
        position: entity.position,
        rotation: entity.rotation,
      },
    });
  }

  removePhysics(entityId: string) {
    this.#postMessage({
      subject: "set_physics",
      data: {
        entityId,
        collider: null,
        position: [0, 0, 0],
        rotation: [0, 0, 0],
      },
    });
  }

  destroy() {
    this.#worker.terminate();
    if (this.#player) this.#player.destroy();
  }

  #postMessage = (message: ToGameMessage) => {
    this.#worker.postMessage(message);
  };
}
