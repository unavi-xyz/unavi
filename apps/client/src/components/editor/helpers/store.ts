import create from "zustand";
import { SceneManager } from "3d";

import { EditorManager, EditorStore } from "./EditorManager";
import { Tool } from "./types";

export const useStore = create<EditorStore>(() => ({
  scene: { instances: {}, assets: {} },
  selected: undefined,
  tool: Tool.translate,
  sceneId: undefined,
  usingGizmo: false,
  previewMode: false,
}));

export const sceneManager = new SceneManager(useStore);
export const editorManager = new EditorManager(useStore);
