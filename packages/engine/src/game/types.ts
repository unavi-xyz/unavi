import { OMICollider, OMIPhysicsBody, Triplet, WorkerMessage } from "../types";

export type ToGameMessage =
  | WorkerMessage<"init_player">
  | WorkerMessage<"jumping", boolean>
  | WorkerMessage<
      "set_physics",
      {
        entityId: string;
        collider: OMICollider | null;
        position: Triplet;
        rotation: Triplet;
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
