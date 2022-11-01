import { GLTF } from "@gltf-transform/core";

import { BoxMesh } from "./BoxMesh";
import { CylinderMesh } from "./CylinderMesh";
import { GLTFMesh } from "./GLTFMesh";
import { PrimitivesMesh } from "./PrimitivesMesh";
import { SphereMesh } from "./SphereMesh";

export interface BaseMeshJSON {
  id: string;
  materialId: string | null;
}

export interface BoxMeshJSON extends BaseMeshJSON {
  type: "Box";
  width: number;
  height: number;
  depth: number;
}

export interface SphereMeshJSON extends BaseMeshJSON {
  type: "Sphere";
  radius: number;
  widthSegments: number;
  heightSegments: number;
}

export interface CylinderMeshJSON extends BaseMeshJSON {
  type: "Cylinder";
  radius: number;
  height: number;
  radialSegments: number;
}

export interface GLTFMeshJSON extends BaseMeshJSON {
  type: "glTF";
  name: string | null;
  uri: string | null;
}

export interface PrimitivesMeshJSON {
  type: "Primitives";
  id: string;
  name: string | null;
  materialId: null;
  primitives: PrimitiveJSON[];
}

export interface PrimitiveJSON {
  id: string;
  materialId: string | null;
  mode: GLTF.MeshPrimitiveMode;
  indicesId: string | null;
  weights: number[];
  morphPositionIds: string[];
  morphNormalIds: string[];
  morphTangentIds: string[];
  POSITION: string | null;
  NORMAL: string | null;
  TANGENT: string | null;
  TEXCOORD_0: string | null;
  TEXCOORD_1: string | null;
  COLOR_0: string | null;
  JOINTS_0: string | null;
  WEIGHTS_0: string | null;
  skin: null | {
    inverseBindMatricesId: string;
    jointIds: string[];
  };
}

export type MeshJSON =
  | BoxMeshJSON
  | SphereMeshJSON
  | CylinderMeshJSON
  | GLTFMeshJSON
  | PrimitivesMeshJSON;

export type Mesh =
  | BoxMesh
  | SphereMesh
  | CylinderMesh
  | GLTFMesh
  | PrimitivesMesh;
