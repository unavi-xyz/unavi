import { SceneMessage } from "../scene";
import { Quad, Triplet, WorkerMessage } from "../types";

export type ToGameMessage =
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
    >;

export type FromGameMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<
      "player_buffers",
      {
        position: Float32Array;
        velocity: Float32Array;
      }
    >;
