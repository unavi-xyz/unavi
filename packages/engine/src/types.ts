import { AnimationAction, Group } from "three";

import { LoadedGLTF } from "./gltf";

export interface IGLTF {
  scene: Group;
  animations: AnimationAction[];
}

export type BoxCollider = {
  type: "box";
  extents?: [number, number, number];
};

export type SphereCollider = {
  type: "sphere";
  radius?: number;
};

export type CapsuleCollider = {
  type: "capsule";
  radius?: number;
  height?: number;
};

export type HullCollider = {
  type: "hull";
};

export type MeshCollider = {
  type: "mesh";
};

export type CompoundCollider = {
  type: "compound";
};

export type OMICollider =
  | BoxCollider
  | SphereCollider
  | CapsuleCollider
  | HullCollider
  | MeshCollider
  | CompoundCollider;

export type OMIPhysicsBody = {
  type: "static" | "dynamic" | "kinematic" | "trigger";
};

export interface UserData {
  OMI_collider?: OMICollider;
  OMI_physics_body?: OMIPhysicsBody;
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

// To loader worker
export type ToLoaderMessage = WorkerMessage<"load_gltf", { uri: string }>;

// From loader worker
export type FromLoaderLoadedGltf = WorkerMessage<"loaded_gltf", LoadedGLTF>;
