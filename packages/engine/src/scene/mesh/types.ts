import { GLTF } from "@gltf-transform/core";

import { BoxMesh } from "./BoxMesh";
import { CylinderMesh } from "./CylinderMesh";
import { GLTFMesh } from "./GLTFMesh";
import { PrimitiveMesh } from "./PrimitiveMesh";
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

export type GLTFMeshJSON = {
  type: "glTF";
  name: string | null;
  uri: string | null;
};

export type PrimitiveMeshJSON = {
  type: "Primitive";
  gltfId: string | null;
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
};

export type MeshJSON =
  | BoxMeshJSON
  | SphereMeshJSON
  | CylinderMeshJSON
  | GLTFMeshJSON
  | PrimitiveMeshJSON;

export type Mesh =
  | BoxMesh
  | SphereMesh
  | CylinderMesh
  | GLTFMesh
  | PrimitiveMesh;
