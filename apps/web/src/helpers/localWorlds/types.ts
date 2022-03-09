import { Scene } from "3d";

export type LocalWorld = {
  id: string;
  name?: string;
  description?: string;
  image?: string;
  scene: Scene;
};
