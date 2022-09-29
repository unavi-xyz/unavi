import {
  AnimationClip,
  BufferAttribute,
  Mesh,
  MeshStandardMaterial,
  Object3D,
} from "three";

export type SceneMap = {
  objects: Map<string, Object3D>;
  materials: Map<string, MeshStandardMaterial>;
  attributes: Map<string, BufferAttribute>;
  animations: Map<string, AnimationClip>;
  colliders: Map<string, Mesh>;
};
