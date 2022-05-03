import { Scene } from "./types";

export const DEFAULT_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],

    parentId: null,
    children: [],
  },
};
