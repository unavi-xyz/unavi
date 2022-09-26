import { Triplet } from "../../types";
import { BoxCollider } from "./BoxCollider";
import { CylinderCollider } from "./CylinderCollider";
import { SphereCollider } from "./SphereCollider";

export type BoxColliderJSON = {
  type: "Box";
  size: Triplet;
};

export type SphereColliderJSON = {
  type: "Sphere";
  radius: number;
};

export type CylinderColliderJSON = {
  type: "Cylinder";
  radius: number;
  height: number;
};

export type ColliderJSON =
  | BoxColliderJSON
  | SphereColliderJSON
  | CylinderColliderJSON;
export type Collider = BoxCollider | SphereCollider | CylinderCollider;
