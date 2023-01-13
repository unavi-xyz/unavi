import { ControlsType } from "../Engine";
import { InputMessage } from "../input/messages";
import { SceneMessage } from "../scene/messages";
import { MessageJSON, Vec2, Vec3, Vec4 } from "../types";

export type ToRenderMessage =
  | InputMessage
  | SceneMessage
  | MessageJSON<"set_canvas", HTMLCanvasElement | OffscreenCanvas>
  | MessageJSON<"set_size", { width: number; height: number }>
  | MessageJSON<"set_pixel_ratio", number>
  | MessageJSON<"player_input_direction", Vec2>
  | MessageJSON<"set_transform_controls_mode", "translate" | "rotate" | "scale">
  | MessageJSON<"set_transform_controls_target", { nodeId: string | null }>
  | MessageJSON<"set_skybox", { uri: string | null }>
  | MessageJSON<"set_controls", ControlsType>
  | MessageJSON<"set_player_arrays", { position: Int32Array; rotation: Int32Array }>;

export type FromRenderMessage =
  | MessageJSON<"ready">
  | MessageJSON<"clicked_node", { nodeId: string | null }>
  | MessageJSON<
      "set_node_transform",
      {
        nodeId: string;
        translation: Vec3;
        rotation: Vec4;
        scale: Vec3;
      }
    >;
