import {
  DracoMeshCompression,
  MeshQuantization,
} from "@gltf-transform/extensions";

import { ColliderExtension } from "./extensions/Collider/ColliderExtension";
import { SpawnPointExtension } from "./extensions/SpawnPoint/SpawnPointExtension";

export const extensions = [
  ColliderExtension,
  SpawnPointExtension,
  DracoMeshCompression,
  MeshQuantization,
];
