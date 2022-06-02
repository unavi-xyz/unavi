import { DEFAULT_TRANSFORM, Entity } from "@wired-xr/scene";

export const ALL_PRESETS: {
  [key: string]: Entity;
} = {
  Box: {
    type: "Box",

    id: "",
    name: "Box",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  Sphere: {
    type: "Sphere",

    id: "",
    name: "Sphere",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  Cylinder: {
    type: "Cylinder",

    id: "",
    name: "Cylinder",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  Model: {
    type: "Model",

    id: "",
    name: "Model",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  PointLight: {
    type: "PointLight",

    id: "",
    name: "Point Light",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
};

export type Preset = keyof typeof ALL_PRESETS;

export const OBJECT_PRESETS: { [key: string]: Preset } = {
  Box: "Box",
  Sphere: "Sphere",
  Cylinder: "Cylinder",
  Model: "Model",
};

export const LIGHTS_PRESETS: { [key: string]: Preset } = {
  ["Point Light"]: "PointLight",
};
