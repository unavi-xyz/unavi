import { ColliderType } from "../../gltf/extensions/Collider/types";
import { Triplet } from "../../types";
import { AutoCollider } from "./AutoCollider";
import { BoxCollider } from "./BoxCollider";
import { CylinderCollider } from "./CylinderCollider";
import { HullCollider } from "./HullCollider";
import { MeshCollider } from "./MeshCollider";
import { SphereCollider } from "./SphereCollider";

interface BaseColliderJSON {
  type: ColliderType | "auto";
}

export interface AutoColliderJSON extends BaseColliderJSON {
  type: "auto";
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
  meshId: string | null;
}

export interface MeshColliderJSON extends BaseColliderJSON {
  type: "mesh";
  meshId: string | null;
}

export type ColliderJSON =
  | AutoColliderJSON
  | BoxColliderJSON
  | SphereColliderJSON
  | CylinderColliderJSON
  | HullColliderJSON
  | MeshColliderJSON;

export type Collider =
  | AutoCollider
  | BoxCollider
  | SphereCollider
  | CylinderCollider
  | HullCollider
  | MeshCollider;
