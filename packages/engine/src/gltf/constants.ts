import {
  EXTTextureWebP,
  KHRDracoMeshCompression,
  KHRMeshQuantization,
  KHRTextureTransform,
} from "@gltf-transform/extensions";

import { BehaviorExtension } from "./extensions/Behavior/BehaviorExtension";
import { ColliderExtension } from "./extensions/Collider/ColliderExtension";
import { SpawnPointExtension } from "./extensions/SpawnPoint/SpawnPointExtension";

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
