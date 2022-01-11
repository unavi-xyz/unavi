import create from "zustand";
import { OBJECTS, OBJ_NAMES, SceneObject } from "3d";

export const useStore = create((set: any) => ({
  scene: {},

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
