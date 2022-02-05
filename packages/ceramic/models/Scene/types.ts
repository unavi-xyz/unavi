import { Triplet } from "@react-three/cannon";
import { SceneObject } from "3d";
export interface Scene {
  name?: string;
  description?: string;
  image?: string;

  spawn?: Triplet;

  objects: SceneObject[];
}
