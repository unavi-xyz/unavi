import { Scene, Transform } from "./types";

export const DEFAULT_TRANSFORM: Transform = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
};

export const DEFAULT_SCENE: Scene = {
  tree: {
    id: "root",
    name: "root",

    transform: DEFAULT_TRANSFORM,
    modules: [],

    parentId: null,
    children: [],
  },
};
