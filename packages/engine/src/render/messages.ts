import { ControlsType } from "../Engine";
import { InputMessage } from "../input/messages";
import { SceneMessage } from "../scene/messages";
import { MessageJSON, Vec2, Vec3, Vec4 } from "../types";

export type ToRenderMessage =
  | InputMessage
  | SceneMessage
  | MessageJSON<"destroy">
  | MessageJSON<"player_input_direction", Vec2>
  | MessageJSON<"set_animations_path", string>
  | MessageJSON<"set_default_avatar", string>
  | MessageJSON<"set_canvas", HTMLCanvasElement | OffscreenCanvas>
  | MessageJSON<"set_controls", ControlsType>
  | MessageJSON<"set_grounded", boolean>
  | MessageJSON<"set_pixel_ratio", number>
  | MessageJSON<"set_size", { width: number; height: number }>
  | MessageJSON<"set_skybox", { uri: string | null }>
  | MessageJSON<"set_transform_controls_mode", "translate" | "rotate" | "scale">
  | MessageJSON<"set_transform_controls_target", { nodeId: string | null }>
  | MessageJSON<
      "set_user_arrays",
      {
        inputRotation: Int16Array;
        userPosition: Int32Array;
        userRotation: Int16Array;
        cameraPosition: Int32Array;
        cameraYaw: Int16Array;
      }
    >
  | MessageJSON<"set_user_avatar", string | null>
  | MessageJSON<"toggle_visuals", boolean>
  // Players
  | MessageJSON<"add_player", { id: number; position: Int32Array; rotation: Int16Array }>
  | MessageJSON<"remove_player", number>
  | MessageJSON<"set_player_avatar", { playerId: number; uri: string | null }>
  | MessageJSON<"set_player_grounded", { playerId: number; grounded: boolean }>
  | MessageJSON<"set_player_name", { playerId: number; name: string | null }>;

export type FromRenderMessage =
  | SceneMessage
  | MessageJSON<"clicked_node", { nodeId: string | null }>
  | MessageJSON<"ready">
  | MessageJSON<
      "set_node_transform",
      { nodeId: string; translation: Vec3; rotation: Vec4; scale: Vec3 }
    >;
