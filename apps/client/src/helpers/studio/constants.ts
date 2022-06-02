import { nanoid } from "nanoid";

import { DEFAULT_TRANSFORM, Scene } from "@wired-xr/scene";

import { deepClone } from "../utils/deepClone";
import { ALL_PRESETS } from "./presets";

export const PROJECT_FILE_NAME = "project.json";

const box = deepClone(ALL_PRESETS["Box"]);
box.id = nanoid();
box.parentId = "root";

const ambientLight = deepClone(ALL_PRESETS["AmbientLight"]);
ambientLight.id = nanoid();
ambientLight.parentId = "root";
//@ts-ignore
ambientLight.props.intensity = 0.1;

const directionalLight = deepClone(ALL_PRESETS["DirectionalLight"]);
directionalLight.id = nanoid();
directionalLight.parentId = "root";
directionalLight.transform.position = [-1, 1, 1];
//@ts-ignore
directionalLight.props.intensity = 0.8;

export const STARTING_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [box, ambientLight, directionalLight],

    props: {},
  },

  assets: {},
};
