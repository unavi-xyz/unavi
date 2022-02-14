import create from "zustand";
import { ASSETS, ASSET_NAMES, EditorObject } from "3d";
import { World } from "ceramic";

export enum TOOLS {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export interface EditorScene {
  [key: string]: EditorObject<ASSET_NAMES>;
}
const editorScene: EditorScene = {};

export const useStore = create((set: any, get: any) => ({
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

  //scene
  scene: editorScene,
  spawn: [0, 0, 0],

  setScene: (newScene: World) => {
    set((state) => {
      state.scene = newScene;
    });
  },

  deleteObject: (object: EditorObject<ASSET_NAMES>) => {
    set((state) => {
      const newScene = { ...state.scene };
      delete newScene[object.id];
      return { scene: newScene };
    });
  },

  addObject: (object: EditorObject<ASSET_NAMES>) => {
    set((state) => {
      const newScene = { ...state.scene };
      newScene[object.id] = object;
      return { scene: newScene };
    });
  },

  newObject: (type: ASSET_NAMES) => {
    const obj: EditorObject<ASSET_NAMES> = new EditorObject(type, {
      ...ASSETS[type].params,
    });

    set((state) => {
      state.addObject(obj);
    });
  },

  save() {
    Object.values(get().scene).forEach((object: EditorObject<ASSET_NAMES>) => {
      object.save();
    });
  },

  load() {
    Object.values(get().scene).forEach((object: EditorObject<ASSET_NAMES>) => {
      object.load();
    });
  },

  toJSON() {
    const map: EditorObject<ASSET_NAMES>[] = Object.values(get().scene);
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
    get().setScene(loaded);
  },
}));
