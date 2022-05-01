import { MutableRefObject } from "react";
import { Group } from "three";
import create, { SetState, GetState } from "zustand";

import { createSceneSlice, ISceneSlice, Scene, TreeObject } from "scene";

import { Tool } from "./types";

export interface IStudioStore extends ISceneSlice {
  tool: Tool;
  usingGizmo: boolean;
  selected: TreeObject | undefined;
  treeRefs: { [id: string]: MutableRefObject<Group | undefined> };

  setRef: (id: string, ref: MutableRefObject<Group | undefined>) => void;
  removeRef: (id: string) => void;
}

export const useStudioStore = create<IStudioStore>(
  (set: SetState<IStudioStore>, get: GetState<IStudioStore>) => ({
    tool: "Translate",
    usingGizmo: false,
    selected: undefined,

    treeRefs: {},

    setRef(id: string, ref: MutableRefObject<Group | undefined>) {
      set({ treeRefs: { ...get().treeRefs, [id]: ref } });
    },

    removeRef(id: string) {
      const newTreeRefs = { ...get().treeRefs };
      delete newTreeRefs[id];
      set({ treeRefs: newTreeRefs });
    },

    ...createSceneSlice(set as any, get),
  })
);
