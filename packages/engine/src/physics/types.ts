import { SceneMessage } from "../scene";
import { Vec3, Vec4, WorkerMessage } from "../types";

export type ToPhysicsMessage =
  | SceneMessage
  | WorkerMessage<"init_player">
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"jumping", boolean>
  | WorkerMessage<
      "set_global_transform",
      {
        nodeId: string;
        position: Vec3;
        rotation: Vec4;
      }
    >
  | WorkerMessage<"sprinting", boolean>
  | WorkerMessage<
      "set_collider_geometry",
      {
        nodeId: string;
        positions: Float32Array;
        indices?: Uint32Array;
      }
    >;

export type FromPhysicsMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<
      "player_buffers",
      {
        position: Int32Array;
        velocity: Int32Array;
      }
    >
  | WorkerMessage<"player_falling", boolean>;
