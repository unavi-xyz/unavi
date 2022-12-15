import { AnimationClip, BufferAttribute, MeshStandardMaterial, Object3D } from "three";

import { AccessorJSON, MeshJSON, NodeJSON } from "../../scene";
import { ObjectQueue } from "./ObjectQueue";

export type SceneMap = {
  accessors: Map<string, AccessorJSON>;
  animations: Map<string, AnimationClip>;
  attributes: Map<string, BufferAttribute>;
  colliders: Map<string, Object3D>;
  nodes: Map<string, NodeJSON>;
  meshes: Map<string, MeshJSON>;
  images: Map<string, ImageBitmap>;
  materials: Map<string, MeshStandardMaterial>;
  objects: Map<string, Object3D>;
  objectQueue: ObjectQueue;
};

export enum UserData {
  isVisual = "isVisual",
  isCompiled = "isCompiled",
}
