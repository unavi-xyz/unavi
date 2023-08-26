import { atom } from "jotai";

import { AtomStore } from "../AtomStore";
import { TreeItem } from "./classes/TreeItem";
import { Tool } from "./types";

class EditorStore extends AtomStore {
  enabled = atom(false);
  items = atom(new Map<bigint, TreeItem>());
  rootId = atom<bigint | undefined>(undefined);
  sceneTreeId = atom<bigint | undefined>(undefined);
  selectedId = atom<bigint | undefined>(undefined);
  tool = atom<Tool>(Tool.Translate);
}

export const editorStore = new EditorStore();
