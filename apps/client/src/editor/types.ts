import { SceneJSON, Triplet } from "@wired-labs/engine";

export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  Node = "node",
}

export type SavedImageJSON = {
  id: string;
  mimeType: string;
};

export type SavedSceneJSON = {
  spawn: Triplet;
  accessors: SceneJSON["accessors"];
  animations: SceneJSON["animations"];
  images: SavedImageJSON[];
  materials: SceneJSON["materials"];
  meshes: SceneJSON["meshes"];
  nodes: SceneJSON["nodes"];
};
