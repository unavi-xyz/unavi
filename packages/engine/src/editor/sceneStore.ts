import { create } from "zustand";

import { Tool } from "./types";

export interface SceneStore {
  enabled: boolean;
  rootId?: string;
  sceneTreeId?: string;
  selectedId?: string;
  tool: Tool;
}

export const useSceneStore = create<SceneStore>(() => ({
  enabled: false,
  rootId: undefined,
  sceneTreeId: undefined,
  selectedId: undefined,
  tool: Tool.Translate,
}));
