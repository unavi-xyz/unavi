import { SceneMessage } from "../scene";
import { WorkerMessage } from "../types";

export type ToGameMessage =
  | SceneMessage
  | WorkerMessage<"init_player">
  | WorkerMessage<"start">
  | WorkerMessage<"stop">
  | WorkerMessage<"jumping", boolean>;

export type FromGameMessage =
  | WorkerMessage<"ready">
  | WorkerMessage<
      "player_buffers",
      {
        position: Float32Array;
        velocity: Float32Array;
      }
    >;
