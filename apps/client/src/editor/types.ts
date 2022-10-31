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
  entities: SceneJSON["entities"];
  materials: SceneJSON["materials"];
  images: SavedImageJSON[];
};
