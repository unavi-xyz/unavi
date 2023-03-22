import {
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
} from "@gltf-transform/extensions";
import {
  AvatarExtension,
  BehaviorExtension,
  ColliderExtension,
  SpawnPointExtension,
} from "@wired-labs/gltf-extensions";

/**
 * List of extensions supported by the engine.
 */
export const extensions = [
  AvatarExtension,
  BehaviorExtension,
  ColliderExtension,
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
  SpawnPointExtension,
];
