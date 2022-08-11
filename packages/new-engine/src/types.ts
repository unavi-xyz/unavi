import { AnimationAction, Group } from "three";

export interface IGLTF {
  scene: Group;
  animations: AnimationAction[];
}

export interface UserData {
  isTreeNode?: boolean;
}

// Messages
type WorkerMessage<S extends string, D> = {
  id?: number;
  subject: S;
  data: D;
};

// To game worker
export type ToGameInitPlayer = WorkerMessage<"init_player", null>;
export type ToGameJump = WorkerMessage<"jumping", boolean>;
export type ToGameMessage = ToGameInitPlayer | ToGameJump;

// From game worker
export type FromGameReady = WorkerMessage<"ready", null>;
export type FromGamePlayerBuffers = WorkerMessage<
  "player_buffers",
  {
    position: Float32Array;
    velocity: Float32Array;
  }
>;
export type FromGameMessage = FromGameReady | FromGamePlayerBuffers;
