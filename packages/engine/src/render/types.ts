import { TypedArray } from "bitecs";

import { WorkerMessage } from "../types";

export interface RenderWorkerOptions {
  skyboxPath?: string;
  controls?: "orbit" | "player";
}

export type LoadSceneData = {
  worldBuffer: ArrayBuffer;
  images: ImageBitmap[];
  accessors: TypedArray[];
};

export type ToRenderMessage =
  | WorkerMessage<"set_canvas", OffscreenCanvas>
  | WorkerMessage<"init", RenderWorkerOptions>
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"destroy">
  | WorkerMessage<"load_scene", LoadSceneData>;

export type FromRenderMessage = WorkerMessage<"ready">;
