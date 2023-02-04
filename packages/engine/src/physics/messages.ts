import { ControlsType } from "../Engine";
import { SceneMessage } from "../scene/messages";
import { MessageJSON } from "../types";

export type ToPhysicsMessage =
  | SceneMessage
  | MessageJSON<"destroy">
  | MessageJSON<"jump">
  | MessageJSON<"set_controls", ControlsType>
  | MessageJSON<"set_sprinting", boolean>
  | MessageJSON<"respawn", null>
  | MessageJSON<
      "set_user_arrays",
      { input: Int16Array; userPosition: Int32Array; cameraYaw: Int16Array }
    >;

export type FromPhysicsMessage = MessageJSON<"ready"> | MessageJSON<"set_grounded", boolean>;
