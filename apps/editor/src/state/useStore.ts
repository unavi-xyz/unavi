import { MutableRefObject } from "react";
import create from "zustand";
import { SceneObject } from "3d";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export const useStore = create((set: any) => ({
  selected: null,
  selectedRef: null,
  tool: TOOLS.translate,
  id: "",

  setId: (id: string) => {
    set((state) => {
      state.id = id;
    });
  },

  setTool: (tool: TOOLS) => {
    set((state) => {
      state.tool = tool;
    });
  },

  setSelected: (
    object: SceneObject | null,
    ref: MutableRefObject<undefined> | null
  ) => {
    set((state) => {
      state.selected = object;
      state.selectedRef = ref;
    });
  },
}));
