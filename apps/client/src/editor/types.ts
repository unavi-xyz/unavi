import { SceneJSON } from "engine";

export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  Node = "node",
}

export type SavedImageJSON = {
  id: string;
  mimeType: string;
};

export type SavedSceneJSON = {
  spawnId: SceneJSON["spawnId"];
  accessors: SceneJSON["accessors"];
  animations: SceneJSON["animations"];
  images: SavedImageJSON[];
  materials: SceneJSON["materials"];
  meshes: SceneJSON["meshes"];
  nodes: SceneJSON["nodes"];
};
