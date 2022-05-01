import { Scene } from "./types";

export const DEFAULT_SCENE: Scene = {
  tree: {
    type: "Group",
    id: "root",
    name: "root",
    children: [],
    params: {
      position: [0, 0, 0],
      rotation: [0, 0, 0],
    },
  },
};
