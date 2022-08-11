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
export type ToGameMessage = ToGameInitPlayer;

// From game worker
export type FromGamePlayerBuffers = WorkerMessage<
  "player_buffers",
  {
    position: Float32Array;
    velocity: Float32Array;
  }
>;
export type FromGameMessage = FromGamePlayerBuffers;
