import create from "zustand";
import { ASSET_NAMES, SceneObject, EditorObject } from "3d";
import { Scene } from "ceramic";

export interface EditorScene {
  [key: string]: EditorObject;
}
const editorScene: EditorScene = {};

export const useScene = create((set: any, get: any) => ({
  scene: editorScene,
  spawn: [0, 0, 0],

  setScene: (newScene: Scene) => {
    set((state) => {
      state.scene = newScene;
    });
  },

  deleteObject: (object: EditorObject) => {
    set((state) => {
      const newScene = { ...state.scene };
      delete newScene[object.id];
      return { scene: newScene };
    });
  },

  addObject: (object: EditorObject) => {
    set((state) => {
      const newScene = { ...state.scene };
      newScene[object.id] = object;
      return { scene: newScene };
    });
  },

  newObject: (type: ASSET_NAMES) => {
    const obj = new EditorObject();
    obj.params.type = type;
    obj.params.name = type;

    set((state) => {
      state.addObject(obj);
    });
  },

  save() {
    Object.values(get().scene).forEach((object: EditorObject) => {
      object.save();
    });
  },

  load() {
    Object.values(get().scene).forEach((object: EditorObject) => {
      object.load();
    });
  },

  toJSON() {
    const map = Object.values(get().scene).map(
      (object: EditorObject) => object.params
    );
    return JSON.stringify(map);
  },

  fromJSON(json: string) {
    const map = JSON.parse(json);
    if (!map) return;

    const loaded: EditorScene = {};
    map.forEach((param: SceneObject) => {
      const obj = new EditorObject(param);
      loaded[obj.id] = obj;
    });

    get().setScene(loaded);
  },
}));
