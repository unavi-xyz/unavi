import { ControlsType } from "../Engine";
import { SceneMessage } from "../scene/messages";
import { MessageJSON } from "../types";

export type ToPhysicsMessage = SceneMessage | MessageJSON<"set_controls", ControlsType>;

export type FromPhysicsMessage =
  | MessageJSON<"ready">
  | MessageJSON<
      "set_player_arrays",
      {
        position: Int32Array;
        rotation: Int32Array;
        input: Int32Array;
      }
    >;
