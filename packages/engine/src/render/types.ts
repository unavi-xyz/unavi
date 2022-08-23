import { TypedArray } from "bitecs";

import { WorkerMessage } from "../types";

export interface RenderWorkerOptions {
  skyboxPath?: string;
  controls?: "orbit" | "player";
  pixelRatio?: number;
  canvasWidth: number;
  canvasHeight: number;
}

export type LoadSceneData = {
  worldBuffer: ArrayBuffer;
  images: ImageBitmap[];
  accessors: TypedArray[];
};

export type FakePointerData = {
  pointerType: string;
  pointerId: number;
  pageX: number;
  pageY: number;
  clientX: number;
  clientY: number;
  button: number;
  ctrlKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
};

export type FakeWheelData = {
  deltaY: number;
};

export type ToRenderMessage =
  | WorkerMessage<"set_canvas", OffscreenCanvas>
  | WorkerMessage<"init", RenderWorkerOptions>
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"destroy">
  | WorkerMessage<"load_scene", LoadSceneData>
  | WorkerMessage<"size", { width: number; height: number }>
  | WorkerMessage<"pointermove", FakePointerData>
  | WorkerMessage<"pointerup", FakePointerData>
  | WorkerMessage<"pointerdown", FakePointerData>
  | WorkerMessage<"pointercancel", FakePointerData>
  | WorkerMessage<"wheel", FakeWheelData>;

export type FromRenderMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<"setPointerCapture", number>
  | WorkerMessage<"releasePointerCapture", number>;
