import { ColliderType } from "../../gltf/extensions/Collider/types";
import { Triplet } from "../../types";
import { BoxCollider } from "./BoxCollider";
import { CylinderCollider } from "./CylinderCollider";
import { HullCollider } from "./HullCollider";
import { MeshCollider } from "./MeshCollider";
import { SphereCollider } from "./SphereCollider";

interface BaseColliderJSON {
  type: ColliderType;
}

export interface BoxColliderJSON extends BaseColliderJSON {
  type: "box";
  size: Triplet;
}

export interface SphereColliderJSON extends BaseColliderJSON {
  type: "sphere";
  radius: number;
}

export interface CylinderColliderJSON extends BaseColliderJSON {
  type: "cylinder";
  radius: number;
  height: number;
}

export interface HullColliderJSON extends BaseColliderJSON {
  type: "hull";
}

export interface MeshColliderJSON extends BaseColliderJSON {
  type: "mesh";
}

export type ColliderJSON =
  | BoxColliderJSON
  | SphereColliderJSON
  | CylinderColliderJSON
  | HullColliderJSON
  | MeshColliderJSON;

export type Collider =
  | BoxCollider
  | SphereCollider
  | CylinderCollider
  | HullCollider
  | MeshCollider;
