import { Variable } from "@wired-labs/gltf-extensions";
import { Engine } from "engine";
import { create } from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;

  canvas: HTMLCanvasElement | null;
  sceneLoaded: boolean;
  isSaving: boolean;
  isPlaying: boolean;
  stopPlaying: () => Promise<void>;

  showColliders: boolean;
  tool: Tool;

  name: string;
  description: string;
  image: string | null;
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

  showColliders: true,
  tool: "translate",

  name: "",
  description: "",
  image: null,
  publicationId: null,

  treeIds: [],
  openIds: [],
  draggingId: null,
  selectedId: null,

  openScriptId: null,
  contextMenuNodeId: null,
  variables: [],
}));
