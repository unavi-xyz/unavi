import { TypedArray } from "bitecs";

import { Entity, Material, WorkerMessage } from "../types";
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
  | WorkerMessage<"set_transform_target", string | null>
  | WorkerMessage<"set_transform_mode", "translate" | "rotate" | "scale">
  | WorkerMessage<"add_entity", Entity>
  | WorkerMessage<
      "set_transform",
      {
        entityId: string;
        position: number[];
        rotation: number[];
        scale: number[];
      }
    >
  | WorkerMessage<
      "set_geometry",
      {
        entityId: string;
        geometry: number[];
      }
    >
  | WorkerMessage<
      "set_material",
      {
        entityId: string;
        materialId: string | null;
      }
    >
  | WorkerMessage<"add_material", Material>
  | WorkerMessage<"edit_material", Material>
  | WorkerMessage<"remove_material", string>
  | WorkerMessage<"remove_entity", string>
  | WorkerMessage<"move_entity", { entityId: string; parentId: string | null }>;

export type FromRenderMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<"clicked_object", string | null>
  | WorkerMessage<
      "set_transform",
      {
        id: string;
        position: number[];
        rotation: number[];
        scale: number[];
      }
    >;
