import { create } from "zustand";

import { TreeItem } from "./classes/TreeItem";

export interface SceneStore {
  enabled: boolean;
  items: Map<bigint, TreeItem>;
  rootId?: bigint;
  sceneTreeId?: bigint;
  selectedId?: bigint;
}

export const useSceneStore = create<SceneStore>(() => ({
  enabled: false,
  items: new Map(),
  rootId: undefined,
  sceneTreeId: undefined,
  selectedId: undefined,
}));
