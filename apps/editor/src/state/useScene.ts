import create from "zustand";
import { OBJECTS, OBJ_NAMES, SceneObject } from "3d";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

interface sceneInterface {
  [key: string]: SceneObject;
}
const typedScene: sceneInterface = {};

export const useScene = create((set: any, get: any) => ({
  scene: typedScene,

  setScene: (newScene: sceneInterface) => {
    set((state) => {
      state.scene = newScene;
    });
  },

  deleteObject: (object: SceneObject) => {
    set((state) => {
      const newScene = { ...state.scene };
      delete newScene[object.id];
      return { scene: newScene };
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

  save() {
    Object.values(get().scene).forEach((object: SceneObject) => {
      object.save();
    });
  },

  load() {
    Object.values(get().scene).forEach((object: SceneObject) => {
      object.load();
    });
  },

  toJSON() {
    return JSON.stringify(get().scene);
  },

  fromJSON(json: string) {
    const objects = JSON.parse(json);
    if (!objects) return;

    const loaded: sceneInterface = {};
    Object.values(objects).forEach((object: any) => {
      const obj = new SceneObject(
        object.name,
        object.position,
        object.rotation,
        object.scale,
        object.color,
        object.opacity
      );
      loaded[obj.id] = obj;
    });

    get().setScene(loaded);
  },
}));
