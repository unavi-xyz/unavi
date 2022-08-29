import { TypedArray } from "bitecs";

import { WorkerMessage } from "../types";
import { RenderWorkerOptions } from "./RenderWorker";

export type LoadSceneData = {
  worldBuffer: ArrayBuffer;
  images: ImageBitmap[];
  accessors: TypedArray[];
};

export type PointerData = {
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
  pointer: {
    x: number;
    y: number;
    button: number;
  };
};

export type WheelData = {
  deltaY: number;
};

export type ToRenderMessage =
  | WorkerMessage<"set_canvas", OffscreenCanvas>
  | WorkerMessage<"init", RenderWorkerOptions>
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"destroy">
  | WorkerMessage<"load_scene", LoadSceneData>
  | WorkerMessage<"update_scene", ArrayBuffer>
  | WorkerMessage<"size", { width: number; height: number }>
  | WorkerMessage<"pointermove", PointerData>
  | WorkerMessage<"pointerup", PointerData>
  | WorkerMessage<"pointerdown", PointerData>
  | WorkerMessage<"pointercancel", PointerData>
  | WorkerMessage<"wheel", WheelData>
  | WorkerMessage<"set_transform_target", number | null>
  | WorkerMessage<"set_transform_mode", "translate" | "rotate" | "scale">;

export type FromRenderMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<"clicked_object", number | null>
  | WorkerMessage<
      "set_transform",
      {
        position: [number, number, number];
        rotation: [number, number, number, number];
        scale: [number, number, number];
      }
    >;
