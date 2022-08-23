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

export type PostMessage<M extends WorkerMessage = WorkerMessage> = (
  message: M,
  transfer?: Transferable[]
) => void;

export type Transferable = ArrayBuffer | MessagePort | ImageBitmap | OffscreenCanvas;

// Messages
export type WorkerMessage<Subject extends string = string, Data = any> = {
  id?: number;
  subject: Subject;
  data: Data;
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
