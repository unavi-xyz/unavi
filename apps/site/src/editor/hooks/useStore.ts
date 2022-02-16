import create from "zustand";
import { ASSETS, ASSET_NAMES, EditorObject } from "3d";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export interface EditorScene {
  [key: string]: EditorObject<ASSET_NAMES>;
}

export const useStore = create((set: any, get: any) => ({
  //editor
  selected: undefined as EditorObject<ASSET_NAMES>,
  usingGizmo: false,
  tool: TOOLS.translate,
  playMode: false,
  sceneId: "",

  setSceneId: (value: string) => {
    set((state) => {
      state.sceneId = value;
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

  //objects
  objects: {} as EditorScene,

  deleteObject: (object: EditorObject<ASSET_NAMES>) => {
    set((state) => {
      const newObjects = { ...state.objects };
      delete newObjects[object.id];
      return { objects: newObjects };
    });
  },

  addObject: (object: EditorObject<ASSET_NAMES>) => {
    const count = Object.values(get().objects as EditorScene).filter(
      (item) => item.instance.type === object.instance.type
    ).length;
    const limit = ASSETS[object.instance.type].limit;
    if (count >= limit && limit >= 0) return false;

    set((state) => {
      const newObjects = { ...state.objects };
      newObjects[object.id] = object;
      return { objects: newObjects };
    });

    return true;
  },

  newObject: (type: ASSET_NAMES) => {
    const obj: EditorObject<ASSET_NAMES> = new EditorObject(type, {
      ...ASSETS[type].params,
    });

    get().addObject(obj);
  },

  save() {
    const objects = Object.values(get().objects);

    objects.forEach((object: EditorObject<ASSET_NAMES>) => {
      object.save();
    });
  },

  load() {
    Object.values(get().objects).forEach(
      (object: EditorObject<ASSET_NAMES>) => {
        object.load();
      }
    );
  },

  toJSON() {
    get().save();
    const map: EditorObject<ASSET_NAMES>[] = Object.values(get().objects);
    return JSON.stringify(map);
  },

  fromJSON(json: string) {
    const map: EditorObject<ASSET_NAMES>[] = JSON.parse(json);
    if (!map) return;
    const loaded: EditorScene = {};
    map.forEach((object) => {
      const obj = new EditorObject<typeof object.instance.type>(
        object.instance.type,
        object.instance.params
      );
      loaded[obj.id] = obj;
    });

    set((state) => {
      state.objects = loaded;
    });
  },
}));
