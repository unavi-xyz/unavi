import { TypedArray } from "bitecs";

import { Entity, Material, Transferable } from "../types";
import { FakeWorker } from "../utils/FakeWorker";
import { RenderWorker } from "./RenderWorker";
import { FromRenderMessage, PointerData, ToRenderMessage } from "./types";

export interface RenderThreadOptions {
  camera: "orbit" | "player";
  skyboxPath?: string;
  enableTransformControls?: boolean;
  preserveDrawingBuffer?: boolean;
}

export class RenderThread {
  ready = false;

  #worker: Worker | FakeWorker;
  #canvas: HTMLCanvasElement;
  #onReady: Array<() => void> = [];
  #onScreenshot: Array<(data: string) => void> = [];

  constructor(
    canvas: HTMLCanvasElement,
    {
      camera,
      skyboxPath,
      enableTransformControls,
      preserveDrawingBuffer,
    }: RenderThreadOptions
  ) {
    this.#canvas = canvas;

    // Render in a worker if browser supports OffscreenCanvas
    // (unless we're in development mode. React 18 dev mode causes issues with transferControlToOffscreen)
    if (
      typeof OffscreenCanvas !== "undefined" &&
      process.env.NODE_ENV !== "development"
    ) {
      console.info(
        "✅ Browser supports OffscreenCanvas. Rendering a in worker..."
      );
      const offscreen = canvas.transferControlToOffscreen();
      this.#worker = new Worker(
        new URL("../workers/Render.worker.ts", import.meta.url),
        {
          type: "module",
        }
      );
      // Initialize worker
      this.#postMessage({ subject: "set_canvas", data: offscreen }, [
        offscreen,
      ]);
    } else {
      console.info(
        "😔 Browser does not support OffscreenCanvas. Rendering in main thread..."
      );
      // Otherwise, render in a fake worker on the main thread
      this.#worker = new FakeWorker();
      const renderWorker = new RenderWorker(
        this.#worker.workerPort.postMessage.bind(this.#worker.workerPort),
        canvas
      );
      this.#worker.workerPort.onmessage =
        renderWorker.onmessage.bind(renderWorker);
    }

    // On message from worker
    this.#worker.onmessage = this.#onmessage.bind(this);

    // Initialize worker
    this.#postMessage({
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
    this.#postMessage({ subject: "take_screenshot", data: null });
    return promise;
  }

  loadScene(
    worldBuffer: ArrayBuffer,
    images: ImageBitmap[],
    accessors: TypedArray[]
  ) {
    this.#postMessage(
      {
        subject: "load_scene",
        data: { worldBuffer, images, accessors },
      },
      [worldBuffer, ...images, ...accessors.map((accessor) => accessor.buffer)]
    );
  }

  addMaterial(material: Material) {
    this.#postMessage({ subject: "add_material", data: material });
  }

  editMaterial(material: Material) {
    this.#postMessage({ subject: "edit_material", data: material });
  }

  removeMaterial(materialId: string) {
    this.#postMessage({ subject: "remove_material", data: materialId });
  }

  addEntity(data: Entity) {
    this.#postMessage({ subject: "add_entity", data });
  }

  setTransform(data: Entity) {
    this.#postMessage({
      subject: "set_transform",
      data: {
        entityId: data.id,
        position: data.position,
        rotation: data.rotation,
        scale: data.scale,
      },
    });
  }

  setGeometry(entityId: string, geometry: number[]) {
    this.#postMessage({
      subject: "set_geometry",
      data: { entityId, geometry },
    });
  }

  setMaterial(entityId: string, materialId: string | null) {
    this.#postMessage({
      subject: "set_material",
      data: { entityId, materialId },
    });
  }

  moveEntity(entityId: string, parentId: string | null) {
    this.#postMessage({
      subject: "move_entity",
      data: { entityId, parentId },
    });
  }

  removeEntity(entityId: string) {
    this.#postMessage({ subject: "remove_entity", data: entityId });
  }

  setPlayerBuffers({
    position,
    velocity,
  }: {
    position: Float32Array;
    velocity: Float32Array;
  }) {
    this.#postMessage(
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

  setPlayerInputVector(data: number[]) {
    this.#postMessage({
      subject: "set_player_input_vector",
      data,
    });
  }

  mouseMove(x: number, y: number) {
    this.#postMessage({
      subject: "mouse_move",
      data: { x, y },
    });
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

  setTransformTarget(target: string | null) {
    this.#postMessage({ subject: "set_transform_target", data: target });
  }

  setTransformMode(mode: "translate" | "rotate" | "scale") {
    this.#postMessage({ subject: "set_transform_mode", data: mode });
  }

  onObjectClick(id: string | null) {}

  onSetTransform(
    id: string,
    position: number[],
    rotation: number[],
    scale: number[]
  ) {}

  #onmessage = (event: MessageEvent<FromRenderMessage>) => {
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
      case "set_transform":
        this.onSetTransform(data.id, data.position, data.rotation, data.scale);
        break;
      case "screenshot":
        this.#onScreenshot.forEach((resolve) => resolve(data));
        this.#onScreenshot = [];
        break;
    }
  };

  #postMessage(message: ToRenderMessage, transfer?: Transferable[]) {
    this.#worker.postMessage(message, transfer);
  }

  onResize() {
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
      data: getPointerData(event, this.#canvas),
    });
  }

  #onPointerUp(event: PointerEvent) {
    this.#canvas.releasePointerCapture(event.pointerId);

    this.#postMessage({
      subject: "pointerup",
      data: getPointerData(event, this.#canvas),
    });
  }

  #onPointerDown(event: PointerEvent) {
    const isPointerLocked = document.pointerLockElement === this.#canvas;
    if (isPointerLocked) return;

    this.#canvas.setPointerCapture(event.pointerId);

    this.#postMessage({
      subject: "pointerdown",
      data: getPointerData(event, this.#canvas),
    });
  }

  #onPointerCancel(event: PointerEvent) {
    this.#postMessage({
      subject: "pointercancel",
      data: getPointerData(event, this.#canvas),
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
