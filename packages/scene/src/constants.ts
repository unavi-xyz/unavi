import { Scene, Transform } from "./types";

export const DEFAULT_TRANSFORM: Transform = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
};

export const EMPTY_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    parentId: null,
    children: [],

    transform: DEFAULT_TRANSFORM,
    props: {},
  },

  assets: {},
};
