import {
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
  KHRXMP,
} from "@gltf-transform/extensions";
import {
  AudioExtension,
  AvatarExtension,
  BehaviorExtension,
  ColliderExtension,
  SpaceExtension,
  SpawnPointExtension,
} from "@unavi/gltf-extensions";

/**
 * List of extensions supported by the engine.
 */
export const extensions = [
  AudioExtension,
  AvatarExtension,
  BehaviorExtension,
  ColliderExtension,
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
  SpawnPointExtension,
  SpaceExtension,
  KHRXMP,
];
