import { GetState, SetState } from "zustand";
import { nanoid } from "nanoid";
import produce from "immer";

import { PrimitiveTreeObject, Scene, TreeObject } from "./types";
import { findObjectById } from "./helpers";
import { Primitive, PRIMITIVES } from "./primitives";
import { DEFAULT_SCENE } from "./constants";

export type StoreSlice<T extends object, E extends object = T> = (
  set: SetState<E extends T ? E : E & T>,
  get: GetState<E extends T ? E : E & T>
) => T;

export interface ISceneSlice {
  id: string;
  name: string;

  scene: Scene;

  addPrimitive: (
    primitive: Primitive,
    parentId?: string
  ) => PrimitiveTreeObject;
  updateObject: (id: string, object: Partial<TreeObject>) => void;
  removeObject: (id: string) => void;
  moveObject: (id: string, parentId: string) => void;
}

export const createSceneSlice: StoreSlice<ISceneSlice> = (set, get) => ({
  id: "",
  name: "",

  scene: DEFAULT_SCENE,

  addPrimitive(primitive: Primitive, parentId = "root") {
    const object: PrimitiveTreeObject<typeof primitive> = {
      type: "Primitive",

      id: nanoid(),
      name: primitive,

      position: [0, 0, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],

      parentId,
      children: [],

      primitive,
      params: PRIMITIVES[primitive].default,
    };

    const scene = produce(get().scene, (draft) => {
      const parent = findObjectById(draft.tree, parentId);
      if (parent) parent.children.push(object);
    });

    set({ scene });

    return object;
  },

  updateObject(id: string, changes: Partial<TreeObject>) {
    const scene = produce(get().scene, (draft) => {
      const found = findObjectById(draft.tree, id);
      if (found) Object.assign(found, changes);
    });

    set({ scene });
  },

  removeObject(id: string) {
    const scene = produce(get().scene, (draft) => {
      const object = findObjectById(draft.tree, id);
      if (!object?.parentId) return;
      const parent = findObjectById(draft.tree, object.parentId);
      if (!parent) return;

      //remove from parent
      const index = parent.children.indexOf(object);
      if (index !== -1) parent.children.splice(index, 1);
    });

    set({ scene });
  },

  moveObject(id: string, parentId: string) {
    const scene = produce(get().scene, (draft) => {
      const object = findObjectById(draft.tree, id);
      if (!object) return;

      if (object?.parentId) {
        //remove from old parent
        const oldParent = findObjectById(draft.tree, object.parentId);
        if (oldParent) {
          oldParent.children = oldParent.children.filter(
            (child) => child.id !== id
          );
        }
      }

      //get new parent
      const parent = findObjectById(draft.tree, parentId);
      if (!parent) return;

      //add to new parent
      object.parentId = parentId;
      parent.children.push(object);
    });

    set({ scene });
  },
});
