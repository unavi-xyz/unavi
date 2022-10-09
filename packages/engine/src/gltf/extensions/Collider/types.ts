import { IProperty, Mesh } from "@gltf-transform/core";

import { Triplet } from "../../../types";

export interface NodeColliderDef {
  collider: number;
}

export interface ColliderDef {
  type: ColliderType;
  size?: Triplet;
  radius?: number;
  height?: number;
  mesh?: number;
}

export interface ColliderExtensionDef {
  colliders: ColliderDef[];
}

export type ColliderType =
  | "box"
  | "capsule"
  | "compound"
  | "cylinder"
  | "hull"
  | "mesh"
  | "sphere";

export interface ICollider extends IProperty {
  type: ColliderType;
  size: Triplet | null;
  radius: number | null;
  height: number | null;
  mesh: Mesh;
}
