import { IProperty, Mesh } from "@gltf-transform/core";

import { Vec3 } from "../../../types";

export interface NodeColliderDef {
  collider: number;
}

export interface ColliderDef {
  type: ColliderType;
  size?: Vec3;
  radius?: number;
  height?: number;
  mesh?: number;
}

export interface ColliderExtensionDef {
  colliders: ColliderDef[];
}

export type ColliderType = "box" | "capsule" | "compound" | "cylinder" | "hull" | "mesh" | "sphere";

export interface ICollider extends IProperty {
  type: ColliderType;
  size: Vec3 | null;
  radius: number | null;
  height: number | null;
  mesh: Mesh;
}
