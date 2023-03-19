import {
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
} from "@gltf-transform/extensions";
import {
  BehaviorExtension,
  ColliderExtension,
  SpawnPointExtension,
} from "@wired-labs/gltf-extensions";

/**
 * List of extensions supported by the engine.
 */
export const extensions = [
  BehaviorExtension,
  ColliderExtension,
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
  SpawnPointExtension,
];
