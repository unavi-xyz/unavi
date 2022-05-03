import { nanoid } from "nanoid";
import { PrimitiveTreeObject, Scene } from "./types";

const DEFAULT_BOX: PrimitiveTreeObject<"Box"> = {
  type: "Primitive",

  id: nanoid(),
  name: "Box",
  children: [],

  primitive: "Box",
  params: {
    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],
  },
};

export const DEFAULT_SCENE: Scene = {
  tree: {
    type: "Group",
    id: "root",
    name: "root",
    children: [DEFAULT_BOX],
    params: {
      position: [0, 0, 0],
      rotation: [0, 0, 0],
    },
  },
};
