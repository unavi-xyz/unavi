import { TypedArray } from "bitecs";

import { SceneMessage } from "../scene";
import { Triplet, WorkerMessage } from "../types";
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
  | SceneMessage
  | WorkerMessage<"set_canvas", OffscreenCanvas>
  | WorkerMessage<"init", RenderWorkerOptions>
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"destroy">
  | WorkerMessage<"size", { width: number; height: number }>
  | WorkerMessage<"pointermove", PointerData>
  | WorkerMessage<"pointerup", PointerData>
  | WorkerMessage<"pointerdown", PointerData>
  | WorkerMessage<"pointercancel", PointerData>
  | WorkerMessage<"wheel", WheelData>
  | WorkerMessage<"set_transform_target", string | null>
  | WorkerMessage<"set_transform_mode", "translate" | "rotate" | "scale">
  | WorkerMessage<"take_screenshot">
  | WorkerMessage<
      "set_player_buffers",
      { position: Float32Array; velocity: Float32Array }
    >
  | WorkerMessage<"set_player_input_vector", number[]>
  | WorkerMessage<
      "mouse_move",
      {
        x: number;
        y: number;
      }
    >;

export type FromRenderMessage =
  | SceneMessage
  | WorkerMessage<"ready">
  | WorkerMessage<"clicked_object", string | null>
  | WorkerMessage<"screenshot", string>;
