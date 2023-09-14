import { create } from "zustand";

import { NodeItem } from "./classes/NodeItem";
import { Tool } from "./types";

export interface SceneStore {
  enabled: boolean;
  items: Map<bigint, NodeItem>;
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
