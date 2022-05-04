import { DEFAULT_TRANSFORM, Entity, IColliderModule, IMeshModule } from "scene";

const BOX_MESH: IMeshModule<"Box"> = {
  id: "",
  type: "Mesh",
  variation: "Box",
  props: {},
};

const BOX_COLLIDER: IColliderModule<"Box"> = {
  id: "",
  type: "Collider",
  variation: "Box",
  props: { args: [1, 1, 1], transform: DEFAULT_TRANSFORM },
};

const SPHERE_MESH: IMeshModule<"Sphere"> = {
  id: "",
  type: "Mesh",
  variation: "Sphere",
  props: {},
};

const SPHERE_COLLIDER: IColliderModule<"Sphere"> = {
  id: "",
  type: "Collider",
  variation: "Sphere",
  props: { args: [1], transform: DEFAULT_TRANSFORM },
};

export const ENTITY_PRESETS: { [key: string]: Entity } = {
  Box: {
    id: "",
    name: "Box",

    transform: DEFAULT_TRANSFORM,
    modules: [BOX_MESH, BOX_COLLIDER],

    parentId: null,
    children: [],
  },
  Sphere: {
    id: "",
    name: "Sphere",

    transform: DEFAULT_TRANSFORM,
    modules: [SPHERE_MESH, SPHERE_COLLIDER],

    parentId: null,
    children: [],
  },
};
