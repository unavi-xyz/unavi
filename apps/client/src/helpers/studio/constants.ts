import { nanoid } from "nanoid";

import { DEFAULT_TRANSFORM, Entity, Scene } from "@wired-xr/scene";

import { ALL_PRESETS } from "./presets";

export const PROJECT_FILE_NAME = "project.json";

const BOX: Entity = ALL_PRESETS["Box"];

BOX.id = nanoid();
BOX.parentId = "root";

export const STARTING_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [BOX],

    props: {},
  },

  assets: {},
};
