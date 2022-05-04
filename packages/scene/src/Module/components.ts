import MeshModule from "./MeshModule/MeshModule";
import ColliderModule from "./ColliderModule/ColliderModule";

import BoxCollider from "./ColliderModule/BoxCollider";
import SphereCollider from "./ColliderModule/SphereCollider";

import BoxMesh from "./MeshModule/BoxMesh";
import SphereMesh from "./MeshModule/SphereMesh";

export type ComponentDefinition = {
  [key: string]: JSX.Element;
};

//module
export const MODULE_COMPONENTS = {
  Mesh: MeshModule,
  Collider: ColliderModule,
};

export type ModuleType = keyof typeof MODULE_COMPONENTS;

//collider
export const COLLIDER_COMPONENTS = {
  Box: BoxCollider,
  Sphere: SphereCollider,
};

export type ColliderType = keyof typeof COLLIDER_COMPONENTS;

//mesh
export const MESH_COMPONENTS = {
  Box: BoxMesh,
  Sphere: SphereMesh,
};

export type MeshType = keyof typeof MESH_COMPONENTS;
