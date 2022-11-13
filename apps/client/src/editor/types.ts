import { AccessorType, SceneJSON } from "@wired-labs/engine";

export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  Node = "node",
}

export type SavedImageJSON = {
  id: string;
  mimeType: string;
};

export type SavedAccessJSON = {
  id: string;
  elementSize: number;
  normalized: boolean;
  type: AccessorType;
};

export type SavedSceneJSON = {
  spawnId: SceneJSON["spawnId"];
  accessors: SavedAccessJSON[];
  animations: SceneJSON["animations"];
  images: SavedImageJSON[];
  materials: SceneJSON["materials"];
  meshes: SceneJSON["meshes"];
  nodes: SceneJSON["nodes"];
};
