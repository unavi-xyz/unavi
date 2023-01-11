import { SceneMessage } from "../scene";
import { Vec3, Vec4, MessageJSON } from "../types";

export type ToPhysicsMessage =
  | SceneMessage
  | MessageJSON<"init_player">
  | MessageJSON<"start">
  | MessageJSON<"stop">
  | MessageJSON<"jumping", boolean>
  | MessageJSON<
      "set_global_transform",
      {
        nodeId: string;
        position: Vec3;
        rotation: Vec4;
      }
    >
  | MessageJSON<"sprinting", boolean>
  | MessageJSON<
      "set_collider_geometry",
      {
        nodeId: string;
        positions: Float32Array;
        indices?: Uint32Array;
      }
    >;

export type FromPhysicsMessage =
  | MessageJSON<"ready">
  | MessageJSON<
      "player_buffers",
      {
        position: Int32Array;
        velocity: Int32Array;
      }
    >
  | MessageJSON<"player_falling", boolean>;
