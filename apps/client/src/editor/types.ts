import { SceneJSON } from "@wired-labs/engine";

export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  Entity = "entity",
}

export type SavedImageJSON = {
  id: string;
  mimeType: string;
};

export type SavedSceneJSON = {
  accessors: SceneJSON["accessors"];
  animations: SceneJSON["animations"];
  entities: SceneJSON["entities"];
  materials: SceneJSON["materials"];
  images: SavedImageJSON[];
};
