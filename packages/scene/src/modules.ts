import MeshModule from "./components/Module/MeshModule/MeshModule";
import ColliderModule from "./components/Module/ColliderModule/ColliderModule";

import BoxCollider from "./components/Module/ColliderModule/BoxCollider";
import SphereCollider from "./components/Module/ColliderModule/SphereCollider";

import BoxMesh from "./components/Module/MeshModule/BoxMesh";
import SphereMesh from "./components/Module/MeshModule/SphereMesh";

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
