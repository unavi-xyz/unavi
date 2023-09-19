import { proxy } from "valtio";

export type SyncedMetadata = {
  title: string;
  description: string;
};

export type SyncedScene = {
  id: string;
  name: string;
};

export enum SyncedNode_Collider_Type {
  NONE = "none",
  BOX = "box",
  SPHERE = "sphere",
  CAPSULE = "capsule",
  CYLINDER = "cylinder",
  MESH = "mesh",
  HULL = "hull",
}

export type SyncedNode_Collider = {
  type: SyncedNode_Collider_Type;
  height: number;
  meshId: string;
  radius: number;
  size: [number, number, number];
};

export enum SyncedNode_RigidBody_Type {
  STATIC = "static",
  DYNAMIC = "dynamic",
}

export type SyncedNode_RigidBody = {
  type: SyncedNode_RigidBody_Type;
};

export type SyncedNode_Extras = Record<string, unknown> & {
  locked: boolean;
};

export type SyncedNode = {
  id: string;
  name: string;

  translation: [number, number, number];
  rotation: [number, number, number, number];
  scale: [number, number, number];

  parentId?: string;
  meshId?: string;

  collider: SyncedNode_Collider;
  rigidBody: SyncedNode_RigidBody;

  extras: SyncedNode_Extras;
};

export type SyncedMesh = {
  id: string;
  name: string;

  indices: number[];
  normals: number[];
  positions: number[];
  uv: number[];
  uv1: number[];
  uv2: number[];
  uv3: number[];
  colors: number[];
  joints: number[];
  weights: number[];
};

export type SyncedStore = {
  initialized: boolean;
  metadata: SyncedMetadata;
  defaultSceneId: string;
  scenes: Record<string, SyncedScene>;
  nodes: Record<string, SyncedNode>;
  meshes: Record<string, SyncedMesh>;
};

export const syncedStore = proxy<SyncedStore>({
  defaultSceneId: "",
  initialized: false,
  meshes: {},
  metadata: {
    description: "",
    title: "",
  },
  nodes: {},
  scenes: {},
});