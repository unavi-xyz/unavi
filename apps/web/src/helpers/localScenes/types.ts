import { Scene } from "3d";

export type LocalScene = {
  id: string;
  name?: string;
  description?: string;
  image?: string;
  scene: Scene;
};
