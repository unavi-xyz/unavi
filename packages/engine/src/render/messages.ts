import { SceneMessage } from "../scene/messages";
import { WorkerMessage } from "../types";

export type ToRenderMessage =
  | SceneMessage
  | WorkerMessage<"set_canvas", HTMLCanvasElement | OffscreenCanvas>;

export type FromRenderMessage = WorkerMessage<"ready">;
