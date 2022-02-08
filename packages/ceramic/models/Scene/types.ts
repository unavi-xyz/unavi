import { Triplet } from "@react-three/cannon";
import { ASSET_NAMES, SceneObject } from "3d";
export interface Scene {
  name?: string;
  description?: string;
  image?: string;

  spawn?: Triplet;

  objects: SceneObject<ASSET_NAMES>[];
}
