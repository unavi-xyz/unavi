import { GLTF, TypedArray } from "@gltf-transform/core";

import { Quad, Triplet, WorkerMessage } from "../types";
import { ColliderJSON } from "./collider/types";
import { MeshJSON } from "./mesh/types";

export type NodeJSON = {
  id: string;
  isInternal: boolean;
  name: string;
  parentId: string;
  position: Triplet;
  rotation: Quad;
  scale: Triplet;
  meshId: string | null;
  collider: ColliderJSON | null;
};

export type TextureJSON = {
  name: string;
  imageId: string | null;
  magFilter: GLTF.TextureMagFilter;
  minFilter: GLTF.TextureMinFilter;
  wrapS: GLTF.TextureWrapMode;
  wrapT: GLTF.TextureWrapMode;
};

export type ImageJSON = {
  id: string;
  isInternal: boolean;
  array: Uint8Array;
  bitmap: ImageBitmap;
  mimeType: string;
};

export type MaterialJSON = {
  id: string;
  isInternal: boolean;
  name: string;
  doubleSided: boolean;
  color: Quad;
  emissive: Triplet;
  roughness: number;
  metalness: number;
  alpha: number;
  alphaCutoff: number;
  alphaMode: "OPAQUE" | "MASK" | "BLEND";
  normalScale: number;
  occlusionStrength: number;
  colorTexture: TextureJSON | null;
  emissiveTexture: TextureJSON | null;
  normalTexture: TextureJSON | null;
  occlusionTexture: TextureJSON | null;
  metallicRoughnessTexture: TextureJSON | null;
};

export type AccessorType =
  | "SCALAR"
  | "VEC2"
  | "VEC3"
  | "VEC4"
  | "MAT2"
  | "MAT3"
  | "MAT4";

export type AccessorJSON = {
  id: string;
  isInternal: boolean;
  array: TypedArray;
  elementSize: number;
  type: AccessorType;
  normalized: boolean;
};

export type AnimationSampler = {
  interpolation: "LINEAR" | "STEP" | "CUBICSPLINE";
  inputId: string;
  outputId: string;
};

export type AnimationChannel = {
  targetId: string;
  path: GLTF.AnimationChannelTargetPath | null;
  sampler: AnimationSampler;
};

export type AnimationJSON = {
  id: string;
  isInternal: boolean;
  name: string;
  channels: AnimationChannel[];
};

export interface SceneJSON {
  spawnId: string | null;
  accessors: AccessorJSON[];
  animations: AnimationJSON[];
  images: ImageJSON[];
  materials: MaterialJSON[];
  meshes: MeshJSON[];
  nodes: NodeJSON[];
}

// Messages
export type SceneMessage =
  | WorkerMessage<
      "load_json",
      {
        scene: Partial<SceneJSON>;
      }
    >
  | WorkerMessage<
      "add_node",
      {
        node: NodeJSON;
      }
    >
  | WorkerMessage<
      "update_node",
      {
        nodeId: string;
        data: Partial<NodeJSON>;
      }
    >
  | WorkerMessage<
      "remove_node",
      {
        nodeId: string;
      }
    >
  | WorkerMessage<
      "add_mesh",
      {
        mesh: MeshJSON;
      }
    >
  | WorkerMessage<
      "update_mesh",
      {
        meshId: string;
        data: Partial<MeshJSON>;
      }
    >
  | WorkerMessage<
      "remove_mesh",
      {
        meshId: string;
      }
    >
  | WorkerMessage<
      "add_material",
      {
        material: MaterialJSON;
      }
    >
  | WorkerMessage<
      "update_material",
      {
        materialId: string;
        data: Partial<MaterialJSON>;
      }
    >
  | WorkerMessage<
      "remove_material",
      {
        materialId: string;
      }
    >;
