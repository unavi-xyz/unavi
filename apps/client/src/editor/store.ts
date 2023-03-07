import { Engine, Variable } from "engine";
import { create } from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;

  canvas: HTMLCanvasElement | null;
  sceneLoaded: boolean;
  isSaving: boolean;
  isPlaying: boolean;
  stopPlaying: () => Promise<void>;

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
  variables: Variable[];
}

export const useEditorStore = create<IEditorStore>(() => ({
  engine: null,
  canvas: null,
  sceneLoaded: false,
  isSaving: false,
  isPlaying: false,
  stopPlaying: async () => {},

  visuals: false,
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
  variables: [],
}));

export interface IExploreStore {
  filter: string;
}

export const useExploreStore = create<IExploreStore>(() => ({
  filter: "",
}));
