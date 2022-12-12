import { Engine } from "../Engine";
import { Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderWorker } from "./RenderWorker";
import { FromRenderMessage, RenderExport, ToRenderMessage } from "./types";

export interface RenderThreadOptions {
  canvas: HTMLCanvasElement;
  engine: Engine;
  camera: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  skyboxPath?: string;
  avatarPath?: string;
  avatarAnimationsPath?: string;
}

/*
 * Acts as an interface between the main thread and the {@link RenderWorker}.
 */
export class RenderThread {
  ready = false;
  worker: Worker | FakeWorker;

  #canvas: HTMLCanvasElement;
  #engine: Engine;
  #onReady: Array<() => void> = [];
  #onExport: Array<(data: RenderExport) => void> = [];

  constructor({
    canvas,
    engine,
    camera,
    enableTransformControls,
    preserveDrawingBuffer,
    skyboxPath,
    avatarPath,
    avatarAnimationsPath,
  }: RenderThreadOptions) {
    this.#canvas = canvas;
    this.#engine = engine;

    // Render in a worker if browser supports OffscreenCanvas
    if (typeof OffscreenCanvas !== "undefined") {
      console.info("✅ Browser supports OffscreenCanvas");

      const offscreen = canvas.transferControlToOffscreen();

      this.worker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
        name: "render",
      });

      // Send canvas
      this.postMessage({ subject: "set_canvas", data: offscreen }, [offscreen]);
    } else {
      console.info("❌ Browser does not support OffscreenCanvas");

      // Render in a fake worker on the main thread
      this.worker = new FakeWorker();

      const renderWorker = new RenderWorker(
        this.worker.workerPort.postMessage.bind(this.worker.workerPort),
        canvas
      );
      this.worker.workerPort.onmessage = renderWorker.onmessage.bind(renderWorker);
    }

    // Handle worker messages
    this.worker.onmessage = this.#onmessage;

    // Initialize worker
    this.postMessage({
      subject: "init",
      data: {
        pixelRatio: window.devicePixelRatio,
        canvasWidth: canvas.clientWidth,
        canvasHeight: canvas.clientHeight,
        avatarAnimationsPath,
        avatarPath,
        camera,
        enableTransformControls,
        preserveDrawingBuffer,
        skyboxPath,
      },
    });

    // Event listeners
    window.addEventListener("resize", this.onResize.bind(this));
  }

  #onmessage = (event: MessageEvent<FromRenderMessage>) => {
    this.#engine.scene.onmessage(event);

    const { subject, data } = event.data;

    switch (subject) {
      case "ready": {
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      }

      case "clicked_object": {
        if (this.onObjectClick) this.onObjectClick(data);
        break;
      }

      case "export": {
        this.#onExport.forEach((resolve) => resolve(data));
        this.#onExport = [];
        break;
      }

      case "set_player_rotation_buffer": {
        this.#engine.networkingInterface.setPlayerRotation(data);
        break;
      }

      case "set_collider_geometry": {
        const transfer: Transferable[] = [data.positions.buffer];
        if (data.indices) transfer.push(data.indices.buffer);

        this.#engine.physicsThread.postMessage(
          {
            subject: "set_collider_geometry",
            data,
          },
          transfer
        );
        break;
      }

      case "player_loaded": {
        this.#engine.networkingInterface.setPlayerLoaded(data);
        break;
      }
    }
  };

  postMessage(message: ToRenderMessage, transfer?: Transferable[]) {
    this.worker.postMessage(message, transfer);
  }

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

      this.#onReady.push(resolve);
    });
  }

  export() {
    const promise = new Promise<RenderExport>((resolve) => {
      this.#onExport.push(resolve);
    });

    this.postMessage({ subject: "prepare_export", data: null });

    return promise;
  }

  setPlayerBuffers({ position, velocity }: { position: Int32Array; velocity: Int32Array }) {
    this.postMessage({
      subject: "set_player_buffers",
      data: {
        position,
        velocity,
      },
    });
  }

  setPlayerInputVector(data: [number, number]) {
    this.postMessage({
      subject: "set_player_input_vector",
      data,
    });
  }

  mouseMove(x: number, y: number) {
    this.postMessage({
      subject: "mouse_move",
      data: { x, y },
    });
  }

  setTransformTarget(target: string | null) {
    this.postMessage({ subject: "set_transform_target", data: target });
  }

  setTransformMode(mode: "translate" | "rotate" | "scale") {
    this.postMessage({ subject: "set_transform_mode", data: mode });
  }

  onObjectClick: ((id: string | null) => void) | null = null;

  onResize() {
    this.postMessage({
      subject: "size",
      data: {
        width: this.#canvas.clientWidth,
        height: this.#canvas.clientHeight,
      },
    });
  }

  destroy() {
    this.worker.postMessage({ subject: "destroy", data: null });
    setTimeout(() => this.worker.terminate());
  }
}
