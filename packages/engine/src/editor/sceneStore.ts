import { create } from "zustand";

import { TreeItem } from "./classes/TreeItem";
import { Tool } from "./types";

export interface SceneStore {
  enabled: boolean;
  items: Map<bigint, TreeItem>;
  rootId?: bigint;
  sceneTreeId?: bigint;
  selectedId?: bigint;
  tool: Tool;
}

export const useSceneStore = create<SceneStore>(() => ({
  enabled: false,
  items: new Map(),
  rootId: undefined,
  sceneTreeId: undefined,
  selectedId: undefined,
  tool: Tool.Translate,
}));
