import { SceneMessage } from "../scene";
import { Quad, Triplet, WorkerMessage } from "../types";

export type ToPhysicsMessage =
  | SceneMessage
  | WorkerMessage<"init_player">
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"jumping", boolean>
  | WorkerMessage<
      "set_global_transform",
      {
        entityId: string;
        position: Triplet;
        rotation: Quad;
      }
    >
  | WorkerMessage<"sprinting", boolean>
  | WorkerMessage<
      "set_collider_geometry",
      {
        entityId: string;
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
