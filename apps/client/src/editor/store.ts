import { Engine } from "engine";
import { create } from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;
  sceneLoaded: boolean;
  isSaving: boolean;

  visuals: boolean;
  tool: Tool;

  name: string;
  description: string;
  publicationId: string | null;

  treeIds: string[];
  openIds: string[];
  draggingId: string | null;
  selectedId: string | null;

  openScriptId: string | null;
  contextMenuNodeId: string | null;
}

export const useEditorStore = create<IEditorStore>(() => ({
  engine: null,
  canvas: null,
  sceneLoaded: false,
  isSaving: false,

  visuals: true,
  tool: "translate",

  name: "",
  description: "",
  publicationId: null,

  treeIds: [],
  openIds: [],
  draggingId: null,
  selectedId: null,

  openScriptId: null,
  contextMenuNodeId: null,
}));
