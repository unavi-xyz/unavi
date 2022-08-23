import { TypedArray } from "bitecs";

import { RenderWorker } from "./render/RenderWorker";
import {
  FakePointerData,
  FromRenderMessage,
  RenderWorkerOptions,
  ToRenderMessage,
} from "./render/types";
import { Transferable } from "./types";
import { FakeWorker } from "./utils/FakeWorker";

export interface RenderThreadOptions extends RenderWorkerOptions {}

export class RenderThread {
  ready = false;

  #worker: Worker | FakeWorker;
  #canvas: HTMLCanvasElement;
  #onReady: Array<() => void> = [];

  constructor(canvas: HTMLCanvasElement, { controls, skyboxPath }: RenderThreadOptions) {
    this.#canvas = canvas;

    // Create worker if browser supports OffscreenCanvas
    // A path to the worker script must be provided
    if (typeof OffscreenCanvas !== "undefined") {
      console.log("🖥️ Rendering with OffscreenCanvas");
      const offscreen = canvas.transferControlToOffscreen();
      this.#worker = new Worker(new URL("./workers/Render.worker.ts", import.meta.url), {
        type: "module",
      });
      // Initialize worker
      this.#postMessage({ subject: "set_canvas", data: offscreen }, [offscreen]);
    } else {
      console.log("🖥️ Rendering on main thread");
      // Otherwise, create a fake worker that runs in the main thread
      this.#worker = new FakeWorker();
      const renderWorker = new RenderWorker(
        this.#worker.workerPort.postMessage.bind(this.#worker.workerPort),
        canvas
      );
      this.#worker.workerPort.onmessage = renderWorker.onmessage;
    }

    // On message from worker
    this.#worker.onmessage = this.#onmessage.bind(this);

    // Initialize worker
    this.#postMessage({
      subject: "init",
      data: {
        controls,
        skyboxPath,
        pixelRatio: window.devicePixelRatio,
        canvasWidth: canvas.clientWidth,
        canvasHeight: canvas.clientHeight,
      },
    });

    // Event listeners\
    window.addEventListener("resize", this.#onResize.bind(this));
    canvas.addEventListener("contextmenu", this.#onContextMenu.bind(this));
    canvas.addEventListener("pointermove", this.#onPointerMove.bind(this));
    canvas.addEventListener("pointerup", this.#onPointerUp.bind(this));
    canvas.addEventListener("pointerdown", this.#onPointerDown.bind(this));
    canvas.addEventListener("pointercancel", this.#onPointerCancel.bind(this));
    canvas.addEventListener("wheel", this.#onWheel.bind(this));
  }

  waitForReady() {
    const promise = new Promise<void>((resolve) => {
      if (this.ready) resolve();
      this.#onReady.push(resolve);
    });
    return promise;
  }

  loadScene(worldBuffer: ArrayBuffer, images: ImageBitmap[], accessors: TypedArray[]) {
    this.#postMessage({ subject: "load_scene", data: { worldBuffer, images, accessors } });
  }

  start() {
    this.#postMessage({ subject: "start", data: null });
  }

  stop() {
    this.#postMessage({ subject: "stop", data: null });
  }

  destroy() {
    this.#worker.postMessage({ subject: "destroy", data: null });
    setTimeout(() => this.#worker.terminate());

    this.#canvas.removeEventListener("contextmenu", this.#onContextMenu);
    this.#canvas.removeEventListener("pointermove", this.#onPointerMove);
    this.#canvas.removeEventListener("pointerup", this.#onPointerUp);
    this.#canvas.removeEventListener("pointerdown", this.#onPointerDown);
    this.#canvas.removeEventListener("pointercancel", this.#onPointerCancel);
    this.#canvas.removeEventListener("wheel", this.#onWheel);
  }

  #onmessage = (event: MessageEvent<FromRenderMessage>) => {
    const { subject, data } = event.data;

    switch (subject) {
      case "ready":
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      case "setPointerCapture":
        this.#canvas.setPointerCapture(data);
        break;
      case "releasePointerCapture":
        this.#canvas.releasePointerCapture(data);
        break;
      default:
        throw new Error(`Unknown message subject: ${subject}`);
    }
  };

  #postMessage(message: ToRenderMessage, transfer?: Transferable[]) {
    this.#worker.postMessage(message, transfer);
  }

  #onResize() {
    this.#postMessage({
      subject: "size",
      data: {
        width: this.#canvas.clientWidth,
        height: this.#canvas.clientHeight,
      },
    });
  }

  #onContextMenu(event: Event) {
    event.preventDefault();
  }

  #onPointerMove(event: PointerEvent) {
    this.#postMessage({
      subject: "pointermove",
      data: getPointerData(event),
    });
  }

  #onPointerUp(event: PointerEvent) {
    this.#postMessage({
      subject: "pointerup",
      data: getPointerData(event),
    });
  }

  #onPointerDown(event: PointerEvent) {
    this.#postMessage({
      subject: "pointerdown",
      data: getPointerData(event),
    });
  }

  #onPointerCancel(event: PointerEvent) {
    this.#postMessage({
      subject: "pointercancel",
      data: getPointerData(event),
    });
  }

  #onWheel(event: WheelEvent) {
    event.preventDefault();
    this.#postMessage({
      subject: "wheel",
      data: {
        deltaY: event.deltaY,
      },
    });
  }
}

function getPointerData(event: PointerEvent): FakePointerData {
  return {
    pointerId: event.pointerId,
    pointerType: event.pointerType,
    clientX: event.clientX,
    clientY: event.clientY,
    pageX: event.pageX,
    pageY: event.pageY,
    button: event.button,
    ctrlKey: event.ctrlKey,
    shiftKey: event.shiftKey,
    metaKey: event.metaKey,
  };
}
