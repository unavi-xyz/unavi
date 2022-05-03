import { MutableRefObject } from "react";
import { Group } from "three";
import create, { SetState, GetState } from "zustand";

import { createSceneSlice, ISceneSlice, Scene, TreeObject } from "scene";

import { Tool } from "./types";

export interface IStudioStore extends ISceneSlice {
  tool: Tool;
  usingGizmo: boolean;
  selectedId: string | undefined;

  openTreeObjects: Set<string>;
  treeRefs: { [id: string]: MutableRefObject<Group | null> };

  setRef: (id: string, ref: MutableRefObject<Group | null>) => void;
  removeRef: (id: string) => void;
}

export const useStudioStore = create<IStudioStore>(
  (set: SetState<IStudioStore>, get: GetState<IStudioStore>) => ({
    tool: "translate",
    usingGizmo: false,
    selectedId: undefined,

    openTreeObjects: new Set(),
    treeRefs: {},

    setRef(id: string, ref: MutableRefObject<Group | null>) {
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
