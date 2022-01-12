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
  roomId: "",

  setRoomId: (id: string) => {
    set((state) => {
      state.roomId = id;
    });
  },

  setTool: (tool: TOOLS) => {
    set((state) => {
      state.tool = tool;
    });
  },

  setSelected: (object: SceneObject, ref: MutableRefObject<undefined>) => {
    set((state) => {
      state.selected = object;
      state.selectedRef = ref;
    });
  },
}));
