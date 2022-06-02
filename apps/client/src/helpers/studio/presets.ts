import { DEFAULT_TRANSFORM, Entity } from "@wired-xr/scene";

export const ENTITY_PRESETS: { [key: string]: Entity } = {
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
};
