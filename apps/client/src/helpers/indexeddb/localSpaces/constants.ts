import { DEFAULT_TRANSFORM, Entity, IMeshModule, Scene } from "scene";
import { nanoid } from "nanoid";

import { ENTITY_PRESETS } from "../../studio/presets";

const BOX: Entity = ENTITY_PRESETS["Box"];

BOX.id = nanoid();
BOX.parentId = "root";
BOX.modules.forEach((module) => {
  module.id = nanoid();
});

export const STARTING_SCENE: Scene = {
  tree: {
    id: "root",
    name: "root",

    transform: DEFAULT_TRANSFORM,
    modules: [],

    parentId: null,
    children: [BOX],
  },
};
