import create from "zustand";
import { SceneObject } from "3d";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export const useStore = create((set: any) => ({
  selected: null,
  usingGizmo: false,
  tool: TOOLS.translate,
  playMode: false,
  id: "",

  setId: (id: string) => {
    set((state) => {
      state.id = id;
    });
  },

  setUsingGizmo: (value: Boolean) => {
    set((state) => {
      state.usingGizmo = value;
    });
  },

  setPlayMode: (value: Boolean) => {
    set((state) => {
      state.playMode = value;
    });
  },

  setTool: (tool: TOOLS) => {
    set((state) => {
      state.tool = tool;
    });
  },

  setSelected: (object: SceneObject) => {
    set((state) => {
      state.selected = object;
    });
  },
}));
