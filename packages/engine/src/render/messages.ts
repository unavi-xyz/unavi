import { SceneMessage } from "../scene/messages";
import { WorkerMessage } from "../types";

export type ToRenderMessage =
  | SceneMessage
  | WorkerMessage<"set_canvas", HTMLCanvasElement | OffscreenCanvas>
  | WorkerMessage<"set_size", { width: number; height: number }>
  | WorkerMessage<"set_pixel_ratio", number>;

export type FromRenderMessage = WorkerMessage<"ready">;
