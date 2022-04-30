import { GetState, SetState } from "zustand";
import { nanoid } from "nanoid";
import produce from "immer";

import { TreeObject } from "./types";
import { findObjectById } from "./helpers";
import { Primitive, PRIMITIVES } from "./primitives";

export type StoreSlice<T extends object, E extends object = T> = (
  set: SetState<E extends T ? E : E & T>,
  get: GetState<E extends T ? E : E & T>
) => T;

export interface ISceneSlice {
  id: string;
  name: string;

  tree: TreeObject<Primitive>;

  addPrimitive: (primitive: Primitive, parentId?: string) => void;
}

export const createSceneSlice: StoreSlice<ISceneSlice> = (set) => ({
  id: "",
  name: "",

  tree: {
    id: "root",
    name: "root",
    parent: undefined,
    children: [],
    primitive: "Group",
    params: undefined as never,
  },

  addPrimitive: (primitive: Primitive, parentId = undefined) => {
    set(
      produce((draft: ISceneSlice) => {
        const object: TreeObject<typeof primitive> = {
          id: nanoid(),
          name: primitive,
          parent: undefined,
          children: [],
          primitive,
          params: PRIMITIVES[primitive].default,
        };

        if (!parentId) {
          draft.tree.children.push(object);
          return;
        }

        //find parent
        const found = findObjectById(draft.tree, parentId);

        if (found) {
          object.parent = found;
          found.children.push(object);
        }
      })
    );
  },
});
