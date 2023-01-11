import { InputMessage } from "../input/messages";
import { SceneMessage } from "../scene/messages";
import { MessageJSON, Vec2 } from "../types";

export type ToRenderMessage =
  | InputMessage
  | SceneMessage
  | MessageJSON<"set_canvas", HTMLCanvasElement | OffscreenCanvas>
  | MessageJSON<"set_size", { width: number; height: number }>
  | MessageJSON<"set_pixel_ratio", number>
  | MessageJSON<"player_input_direction", Vec2>;

export type FromRenderMessage =
  | MessageJSON<"ready">
  | MessageJSON<"clicked_node", { id: string | null }>;
