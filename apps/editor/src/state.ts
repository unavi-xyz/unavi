import { MutableRefObject } from "react";
import create from "zustand";

import { OBJECTS, OBJ_NAMES, SceneObject } from "3d";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export const useStore = create((set: any) => ({
  scene: {},
  selected: null,
  selectedRef: null,
  tool: TOOLS.translate,

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

  addObject: (object: SceneObject) => {
    set((state) => {
      const newScene = { ...state.scene };
      newScene[object.id] = object;
      return { scene: newScene };
    });
  },

  newObject: (name: OBJ_NAMES) => {
    const obj = OBJECTS[name].clone();
    set((state) => {
      state.addObject(obj);
    });
  },
}));
