import { GLTF, TypedArray } from "@gltf-transform/core";

import { SceneMessage } from "../scene";
import { Quad, Triplet, WorkerMessage } from "../types";
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

export type Plugin<T extends WorkerMessage> = {
  onmessage: (message: MessageEvent<T>) => void;
  animate?: (delta: number) => void;
  destroy?: () => void;
};

export type WheelData = {
  deltaY: number;
};

export type RenderExport = Array<{
  entityId: string;
  attributeName: string;
  array: TypedArray;
  normalized: boolean;
  type: GLTF.AccessorType;
}>;

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
  | WorkerMessage<"prepare_export">
  | WorkerMessage<
      "set_player_buffers",
      { position: Int32Array; velocity: Int32Array }
    >
  | WorkerMessage<"set_player_input_vector", [number, number]>
  | WorkerMessage<
      "mouse_move",
      {
        x: number;
        y: number;
      }
    >
  | WorkerMessage<
      "show_visuals",
      {
        visible: boolean;
      }
    >
  | WorkerMessage<"player_joined", string>
  | WorkerMessage<"player_left", string>
  | WorkerMessage<
      "set_player_location",
      {
        playerId: string;
        location: [number, number, number, number, number, number, number];
      }
    >
  | WorkerMessage<
      "set_player_falling_state",
      {
        playerId: string;
        isFalling: boolean;
      }
    >
  | WorkerMessage<
      "set_player_avatar",
      {
        playerId: string;
        avatar: string | null;
      }
    >;

export type FromRenderMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<"clicked_object", string | null>
  | WorkerMessage<"export", RenderExport>
  | WorkerMessage<
      "set_transform",
      {
        entityId: string;
        position: Triplet;
        rotation: Quad;
        scale: Triplet;
      }
    >
  | WorkerMessage<
      "set_global_transform",
      {
        entityId: string;
        position: Triplet;
        rotation: Quad;
      }
    >
  | WorkerMessage<"set_player_rotation_buffer", Int32Array>;
