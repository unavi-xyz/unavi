import { PrimitiveTreeObject, Scene } from "scene";
import { nanoid } from "nanoid";

const BOX: PrimitiveTreeObject<"Box"> = {
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

export const STARTING_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],

    parentId: null,
    children: [BOX],
  },
};
