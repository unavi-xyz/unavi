import { Scene } from "../scene";
import { Transferable, Triplet } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderWorker } from "./RenderWorker";
import { FromRenderMessage, PointerData, ToRenderMessage } from "./types";

export interface RenderThreadOptions {
  canvas: HTMLCanvasElement;
  camera: "orbit" | "player";
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
  scene: Scene;
  skyboxPath?: string;
}

/*
 * RenderThread acts as an interface between the main thread and the {@link RenderWorker}.
 */
export class RenderThread {
  ready = false;
  worker: Worker | FakeWorker;

  #canvas: HTMLCanvasElement;
  #scene: Scene;
  #onReady: Array<() => void> = [];
  #onScreenshot: Array<(data: string) => void> = [];

  constructor({
    canvas,
    camera,
    enableTransformControls,
    preserveDrawingBuffer,
    scene,
    skyboxPath,
  }: RenderThreadOptions) {
    this.#canvas = canvas;
    this.#scene = scene;

    // Render in a worker if browser supports OffscreenCanvas
    // (unless we're in development mode. React 18 dev mode causes issues with transferControlToOffscreen)
    if (
      typeof OffscreenCanvas !== "undefined" &&
      process.env.NODE_ENV !== "development"
    ) {
      console.info("✅ Browser supports OffscreenCanvas");
      const offscreen = canvas.transferControlToOffscreen();

      this.worker = new Worker(
        new URL("../workers/Render.worker.ts", import.meta.url),
        {
          type: "module",
        }
      );

      // Send canvas
      this.postMessage({ subject: "set_canvas", data: offscreen }, [offscreen]);
    } else {
      console.info("😔 Browser does not support OffscreenCanvas");

      // Render in a fake worker on the main thread
      this.worker = new FakeWorker();
      const renderWorker = new RenderWorker(
        this.worker.workerPort.postMessage.bind(this.worker.workerPort),
        canvas
      );
      this.worker.workerPort.onmessage =
        renderWorker.onmessage.bind(renderWorker);
    }

    // Handle worker messages
    this.worker.onmessage = this.#onmessage.bind(this);

    // Initialize worker
    this.postMessage({
      subject: "init",
      data: {
        pixelRatio: window.devicePixelRatio,
        canvasWidth: canvas.clientWidth,
        canvasHeight: canvas.clientHeight,
        camera,
        skyboxPath,
        enableTransformControls,
        preserveDrawingBuffer,
      },
    });

    // Event listeners
    window.addEventListener("resize", this.onResize.bind(this));
    canvas.addEventListener("contextmenu", this.#onContextMenu.bind(this));
    canvas.addEventListener("pointermove", this.#onPointerMove.bind(this));
    canvas.addEventListener("pointerup", this.#onPointerUp.bind(this));
    canvas.addEventListener("pointerdown", this.#onPointerDown.bind(this));
    canvas.addEventListener("pointercancel", this.#onPointerCancel.bind(this));
    canvas.addEventListener("wheel", this.#onWheel.bind(this));
  }

  #onmessage = (event: MessageEvent<FromRenderMessage>) => {
    this.#scene.onmessage(event as any);

    const { subject, data } = event.data;

    switch (subject) {
      case "ready":
        this.ready = true;
        this.#onReady.forEach((resolve) => resolve());
        this.#onReady = [];
        break;
      case "clicked_object":
        this.onObjectClick(data);
        break;
      case "screenshot":
        this.#onScreenshot.forEach((resolve) => resolve(data));
        this.#onScreenshot = [];
        break;
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
    const promise = new Promise<void>((resolve) => {
      if (this.ready) resolve();
      this.#onReady.push(resolve);
    });
    return promise;
  }

  takeScreenshot() {
    const promise = new Promise<string>((resolve) =>
      this.#onScreenshot.push(resolve)
    );
    this.postMessage({ subject: "take_screenshot", data: null });
    return promise;
  }

  setPlayerBuffers({
    position,
    velocity,
  }: {
    position: Float32Array;
    velocity: Float32Array;
  }) {
    this.postMessage(
      {
        subject: "set_player_buffers",
        data: {
          position,
          velocity,
        },
      },
      [position, velocity]
    );
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

  onObjectClick(id: string | null) {}

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

    this.#canvas.removeEventListener("contextmenu", this.#onContextMenu);
    this.#canvas.removeEventListener("pointermove", this.#onPointerMove);
    this.#canvas.removeEventListener("pointerup", this.#onPointerUp);
    this.#canvas.removeEventListener("pointerdown", this.#onPointerDown);
    this.#canvas.removeEventListener("pointercancel", this.#onPointerCancel);
    this.#canvas.removeEventListener("wheel", this.#onWheel);
  }

  #onContextMenu(event: Event) {
    event.preventDefault();
  }

  #onPointerMove(event: PointerEvent) {
    this.postMessage({
      subject: "pointermove",
      data: getPointerData(event, this.#canvas),
    });
  }

  #onPointerUp(event: PointerEvent) {
    this.#canvas.releasePointerCapture(event.pointerId);

    this.postMessage({
      subject: "pointerup",
      data: getPointerData(event, this.#canvas),
    });
  }

  #onPointerDown(event: PointerEvent) {
    const isPointerLocked = document.pointerLockElement === this.#canvas;
    if (isPointerLocked) return;

    this.#canvas.setPointerCapture(event.pointerId);

    this.postMessage({
      subject: "pointerdown",
      data: getPointerData(event, this.#canvas),
    });
  }

  #onPointerCancel(event: PointerEvent) {
    this.postMessage({
      subject: "pointercancel",
      data: getPointerData(event, this.#canvas),
    });
  }

  #onWheel(event: WheelEvent) {
    event.preventDefault();
    this.postMessage({
      subject: "wheel",
      data: {
        deltaY: event.deltaY,
      },
    });
  }
}

function getPointerData(
  event: PointerEvent,
  canvas: HTMLCanvasElement
): PointerData {
  let pointer;
  if (canvas.ownerDocument.pointerLockElement) {
    pointer = {
      x: 0,
      y: 0,
      button: event.button,
    };
  } else {
    const rect = canvas.getBoundingClientRect();
    pointer = {
      x: ((event.clientX - rect.left) / rect.width) * 2 - 1,
      y: (-(event.clientY - rect.top) / rect.height) * 2 + 1,
      button: event.button,
    };
  }

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
    pointer,
  };
}
