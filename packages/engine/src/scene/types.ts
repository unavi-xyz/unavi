import { Triplet, WorkerMessage } from "../types";
import { BoxMesh } from "./BoxMesh";
import { CylinderMesh } from "./CylinderMesh";
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

export type EntityJSON = {
  id: string;
  name: string;
  parentId: string;
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
  mesh: MeshJSON | null;
  materialId: string | null;
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
