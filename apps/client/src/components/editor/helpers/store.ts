import create from "zustand";
import { SceneManager } from "3d";

import { EditorManager, EditorStore } from "./classes/EditorManager";

export const useStore = create<EditorStore>(() => ({
  scene: { instances: {}, assets: {}, materials: {} },
  selected: undefined,
  tool: "translate",
  sceneId: undefined,
  usingGizmo: false,
  previewMode: false,
  debugMode: true,
}));

export const sceneManager = new SceneManager(useStore);
export const editorManager = new EditorManager(useStore);
