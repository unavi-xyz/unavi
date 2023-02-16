import {
  DracoMeshCompression,
  MeshQuantization,
  TextureTransform,
} from "@gltf-transform/extensions";

import { BehaviorExtension } from "./extensions/Behavior/BehaviorExtension";
import { ColliderExtension } from "./extensions/Collider/ColliderExtension";
import { SpawnPointExtension } from "./extensions/SpawnPoint/SpawnPointExtension";

export const extensions = [
  BehaviorExtension,
  ColliderExtension,
  DracoMeshCompression,
  MeshQuantization,
  SpawnPointExtension,
  TextureTransform,
];
