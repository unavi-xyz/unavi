import { Triplet, WorkerMessage } from "../types";
import { BoxCollider } from "./BoxCollider";
import { BoxMesh } from "./BoxMesh";
import { CylinderCollider } from "./CylinderCollider";
import { CylinderMesh } from "./CylinderMesh";
import { SphereCollider } from "./SphereCollider";
import { SphereMesh } from "./SphereMesh";

export type BoxMeshJSON = {
  type: "Box";
  width: number;
  height: number;
  depth: number;
};

export type SphereMeshJSON = {
  type: "Sphere";
  radius: number;
  widthSegments: number;
  heightSegments: number;
};

export type CylinderMeshJSON = {
  type: "Cylinder";
  radius: number;
  height: number;
  radialSegments: number;
};

export type MeshJSON = BoxMeshJSON | SphereMeshJSON | CylinderMeshJSON;
export type Mesh = BoxMesh | SphereMesh | CylinderMesh;

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

export type EntityJSON = {
  id: string;
  name: string;
  parentId: string;
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
  mesh: MeshJSON | null;
  materialId: string | null;
  collider: ColliderJSON | null;
};

export type MaterialJSON = {
  id: string;
  name: string;
  color: Triplet;
  roughness: number;
  metalness: number;
};

export type SceneJSON = {
  entities: EntityJSON[];
  materials: MaterialJSON[];
};

// Messages
export type SceneMessage =
  | WorkerMessage<
      "load_json",
      {
        scene: SceneJSON;
      }
    >
  | WorkerMessage<
      "add_entity",
      {
        entity: EntityJSON;
      }
    >
  | WorkerMessage<
      "remove_entity",
      {
        entityId: string;
      }
    >
  | WorkerMessage<
      "update_entity",
      {
        entityId: string;
        data: Partial<EntityJSON>;
      }
    >
  | WorkerMessage<
      "update_global_transform",
      {
        entityId: string;
        position: Triplet;
        quaternion: [number, number, number, number];
      }
    >
  | WorkerMessage<
      "add_material",
      {
        material: MaterialJSON;
      }
    >
  | WorkerMessage<
      "remove_material",
      {
        materialId: string;
      }
    >
  | WorkerMessage<
      "update_material",
      {
        materialId: string;
        data: Partial<MaterialJSON>;
      }
    >;
