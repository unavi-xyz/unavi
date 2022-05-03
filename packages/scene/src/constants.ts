import { nanoid } from "nanoid";
import { PrimitiveTreeObject, Scene } from "./types";

const DEFAULT_BOX: PrimitiveTreeObject<"Box"> = {
  type: "Primitive",

  id: nanoid(),
  name: "Box",

  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],

  primitive: "Box",
  params: {},

  parentId: "root",
  children: [],
};

export const DEFAULT_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],

    parentId: null,
    children: [DEFAULT_BOX],
  },
};
