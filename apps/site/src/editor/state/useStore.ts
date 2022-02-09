import create from "zustand";
import { ASSET_NAMES, EditorObject } from "3d";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export const useStore = create((set: any) => ({
  selected: undefined as EditorObject<ASSET_NAMES>,
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

  setSelected: (object: EditorObject<ASSET_NAMES>) => {
    set((state) => {
      state.selected = object;
    });
  },
}));
