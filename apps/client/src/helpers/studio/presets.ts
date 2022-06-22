import { DEFAULT_TRANSFORM, Entity } from "@wired-xr/engine";

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

  PointLight: {
    type: "PointLight",

    id: "",
    name: "Point Light",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  AmbientLight: {
    type: "AmbientLight",

    id: "",
    name: "Ambient Light",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  DirectionalLight: {
    type: "DirectionalLight",

    id: "",
    name: "Directional Light",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  SpotLight: {
    type: "SpotLight",

    id: "",
    name: "Spot Light",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },

  Group: {
    type: "Group",

    id: "",
    name: "Group",

    transform: DEFAULT_TRANSFORM,

    parentId: null,
    children: [],

    props: {},
  },
  Text: {
    type: "Text",

    id: "",
    name: "Text",

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
  Spawn: {
    type: "Spawn",

    id: "",
    name: "Spawn",

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
};

export const LIGHTS_PRESETS: { [key: string]: Preset } = {
  ["Point Light"]: "PointLight",
  ["Ambient Light"]: "AmbientLight",
  ["Directional Light"]: "DirectionalLight",
  ["Spot Light"]: "SpotLight",
};

export const SPECIALS_PRESETS: { [key: string]: Preset } = {
  Group: "Group",
  Text: "Text",
  Model: "Model",
  Spawn: "Spawn",
};
