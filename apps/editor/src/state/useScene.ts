import create from "zustand";
import { ASSET_NAMES, Params, SceneObject } from "3d";

export interface IScene {
  [key: string]: SceneObject;
}
const typedScene: IScene = {};

export const useScene = create((set: any, get: any) => ({
  scene: typedScene,

  setScene: (newScene: IScene) => {
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

  newObject: (type: ASSET_NAMES) => {
    const obj = new SceneObject();
    obj.params.type = type;
    obj.params.name = type;

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
    const map = Object.values(get().scene).map(
      (object: SceneObject) => object.params
    );
    return JSON.stringify(map);
  },

  fromJSON(json: string) {
    const map = JSON.parse(json);
    if (!map) return;

    const loaded: IScene = {};
    map.forEach((param: Params) => {
      const obj = new SceneObject(param);
      loaded[obj.id] = obj;
    });

    get().setScene(loaded);
  },
}));
