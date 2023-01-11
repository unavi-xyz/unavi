import { InputMessage } from "../input/messages";
import { SceneMessage } from "../scene/messages";
import { Vec2, WorkerMessage } from "../types";

export type ToRenderMessage =
  | InputMessage
  | SceneMessage
  | WorkerMessage<"set_canvas", HTMLCanvasElement | OffscreenCanvas>
  | WorkerMessage<"set_size", { width: number; height: number }>
  | WorkerMessage<"set_pixel_ratio", number>
  | WorkerMessage<"player_input_direction", Vec2>;

export type FromRenderMessage = WorkerMessage<"ready">;
