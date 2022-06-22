import { nanoid } from "nanoid";

import { DEFAULT_TRANSFORM, Entity, Scene } from "@wired-xr/engine";

import { deepClone } from "../utils/deepClone";
import { ALL_PRESETS } from "./presets";

export const PROJECT_FILE_NAME = "project.json";

const box: Entity<"Box"> = deepClone(ALL_PRESETS["Box"]);
box.id = nanoid();
box.parentId = "root";
box.props.width = 10;
box.props.height = 0.5;
box.props.depth = 10;
box.transform.position = [0, -0.75, 0];

const ambientLight: Entity<"AmbientLight"> = deepClone(
  ALL_PRESETS["AmbientLight"]
);
ambientLight.id = nanoid();
ambientLight.parentId = "root";
ambientLight.props.intensity = 0.1;

const directionalLight: Entity<"DirectionalLight"> = deepClone(
  ALL_PRESETS["DirectionalLight"]
);
directionalLight.id = nanoid();
directionalLight.parentId = "root";
directionalLight.transform.position = [-1, 1, 1];
directionalLight.props.intensity = 0.8;

const spawn: Entity<"Spawn"> = deepClone(ALL_PRESETS["Spawn"]);
spawn.id = nanoid();
spawn.parentId = "root";
spawn.transform.position = [0, 2, 0];

export const STARTING_SCENE: Scene = {
  tree: {
    type: "Group",

    id: "root",
    name: "root",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [box, ambientLight, directionalLight, spawn],

    props: {},
  },

  assets: {},
};
