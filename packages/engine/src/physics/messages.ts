import { DebugRenderBuffers } from "@dimforge/rapier3d";

import { SceneMessage } from "../scene/messages";
import { ControlsType, MessageJSON } from "../types";

export type ToPhysicsMessage =
  | SceneMessage
  | MessageJSON<"destroy">
  | MessageJSON<"jump">
  | MessageJSON<"start">
  | MessageJSON<"stop">
  | MessageJSON<"set_controls", ControlsType>
  | MessageJSON<"set_sprinting", boolean>
  | MessageJSON<"respawn", null>
  | MessageJSON<"toggle_visuals", boolean>
  | MessageJSON<
      "set_user_arrays",
      {
        input: Int16Array;
        userPosition: Int32Array;
        cameraYaw: Int16Array;
      }
    >;

export type FromPhysicsMessage =
  | MessageJSON<"ready">
  | MessageJSON<"set_grounded", boolean>
  | MessageJSON<"set_debug_buffers", DebugRenderBuffers>;
