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
  updateObject: (id: string, object: Partial<TreeObject["params"]>) => void;
}

export const createSceneSlice: StoreSlice<ISceneSlice> = (set, get) => ({
  id: "",
  name: "",

  scene: DEFAULT_SCENE,

  addPrimitive(primitive: Primitive, parentId = undefined) {
    const object: PrimitiveTreeObject<typeof primitive> = {
      type: "Primitive",

      id: nanoid(),
      name: primitive,
      children: [],

      primitive,
      params: PRIMITIVES[primitive].default,
    };

    const scene = produce(get().scene, (draft: ISceneSlice["scene"]) => {
      if (!parentId) {
        draft.tree.children.push(object);
        return;
      }

      //find parent
      const found = findObjectById(draft.tree, parentId);
      if (found) found.children.push(object);
    });

    set({ scene });

    return object;
  },

  updateObject(id: string, params: Partial<TreeObject["params"]>) {
    set(
      produce((draft: ISceneSlice) => {
        const found = findObjectById(draft.scene.tree, id);
        if (found) Object.assign(found.params, params);
      })
    );
  },
});
