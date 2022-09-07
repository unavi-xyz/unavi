import { Entity } from "@wired-labs/engine";

export const emptyTree: { [id: string]: Entity } = {
  root: {
    id: "root",
    name: "root",
    type: "Group",
    parent: null,
    children: [],
    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],
  },
};
