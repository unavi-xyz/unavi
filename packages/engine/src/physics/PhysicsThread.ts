import { Engine } from "../Engine";
import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { PhysicsWorker } from "./PhysicsWorker";
import { Player } from "./Player";
import { FromPhysicsMessage, ToPhysicsMessage } from "./types";

export interface PhysicsThreadOptions {
  canvas: HTMLCanvasElement;
  engine: Engine;
}

/*
 * Acts as an interface between the main thread and the {@link PhysicsWorker}.
 */
export class PhysicsThread {
  #worker = new FakeWorker();
  // #worker = new Worker(new URL("./worker.ts", import.meta.url), {
  //   type: "module",
  //   name: "physics",
  // });

  ready = false;
  #readyListeners: Array<() => void> = [];

  #canvas: HTMLCanvasElement;
  #engine: Engine;

  #player: Player | null = null;

  constructor({ canvas, engine }: PhysicsThreadOptions) {
    this.#canvas = canvas;
    this.#engine = engine;

    const physicsWorker = new PhysicsWorker(
      this.#worker.workerPort.postMessage.bind(this.#worker.workerPort)
    );
    this.#worker.workerPort.onmessage =
      physicsWorker.onmessage.bind(physicsWorker);

    this.#worker.onmessage = this.#onmessage;
  }

  #onmessage = (event: MessageEvent<FromPhysicsMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;
        this.#readyListeners.forEach((listener) => listener());
        break;
      }

      case "player_buffers": {
        this.#engine.renderThread.setPlayerBuffers(data);
        this.#engine.networkingInterface.setPlayerPosition(data.position);
        break;
      }

      case "player_falling": {
        this.#engine.networkingInterface.setFallState(data);
      }
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
    this.#player = new Player(this.#canvas, this.#engine.renderThread, this);
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
