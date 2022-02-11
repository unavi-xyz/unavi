import { Triplet } from "@react-three/cannon";
import { ASSET_NAMES, SceneObject } from "3d";

export interface World {
  name?: string;
  description?: string;
  image?: string;

  spawn?: Triplet;

  scene: SceneObject<ASSET_NAMES>[];
}
