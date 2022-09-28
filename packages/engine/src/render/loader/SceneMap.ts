import {
  AnimationClip,
  BufferAttribute,
  Mesh,
  MeshStandardMaterial,
  Object3D,
} from "three";

/*
 * Maps scene items to their corresponding Three.js objects.
 */
export class SceneMap {
  objects = new Map<string, Object3D>();
  materials = new Map<string, MeshStandardMaterial>();
  attributes = new Map<string, BufferAttribute>();
  animations = new Map<string, AnimationClip>();
  colliders = new Map<string, Mesh>();
}
