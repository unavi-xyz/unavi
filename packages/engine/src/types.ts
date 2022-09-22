export type PostMessage<M extends WorkerMessage = WorkerMessage> = (
  message: M,
  transfer?: Transferable[]
) => void;

export type Transferable =
  | ArrayBuffer
  | MessagePort
  | ImageBitmap
  | OffscreenCanvas;

export type WorkerMessage<Subject extends string = string, Data = any> = {
  subject: Subject;
  data: Data;
};

export type Triplet = [number, number, number];

// Physics
export type BoxCollider = {
  type: "box";
  extents?: Triplet;
};

export type SphereCollider = {
  type: "sphere";
  radius?: number;
};

export type CylinderCollider = {
  type: "cylinder";
  radius?: number;
  height?: number;
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
  | CylinderCollider
  | CapsuleCollider
  | HullCollider
  | MeshCollider
  | CompoundCollider;

export type OMIPhysicsBody = {
  type: "static" | "dynamic" | "kinematic" | "trigger";
};
