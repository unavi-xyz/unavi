import { ControlsType } from "../Engine";
import { SceneMessage } from "../scene/messages";
import { MessageJSON } from "../types";

export type ToPhysicsMessage =
  | SceneMessage
  | MessageJSON<"set_controls", ControlsType>
  | MessageJSON<"set_sprinting", boolean>;

export type FromPhysicsMessage =
  | MessageJSON<"ready">
  | MessageJSON<
      "set_player_arrays",
      { input: Int16Array; position: Int32Array; rotation: Int32Array }
    >;
