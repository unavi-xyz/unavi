import { nanoid } from "nanoid";

import { DEFAULT_TRANSFORM, Entity, Scene } from "@wired-xr/scene";

import { ALL_PRESETS } from "./presets";

export const PROJECT_FILE_NAME = "project.json";

const box = ALL_PRESETS["Box"];
box.id = nanoid();
box.parentId = "root";

const ambientLight = ALL_PRESETS["AmbientLight"];
ambientLight.id = nanoid();
ambientLight.parentId = "root";

export const STARTING_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [box, ambientLight],

    props: {},
  },

  assets: {},
};
