import { Engine } from "engine";
import create from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;
  sceneLoaded: boolean;
  changesToSave: boolean;
  isSaving: boolean;

  canvas: HTMLCanvasElement | null;
  preview: boolean;
  selectedId: string | null;

  name: string;
  description: string;
  publicationId: string | null;

  visuals: boolean;
  tool: Tool;

  treeIds: string[];
  openIds: string[];
  draggingId: string | null;
}

export const useEditorStore = create<IEditorStore>(() => ({
  engine: null,
  sceneLoaded: false,
  changesToSave: false,
  isSaving: false,

  canvas: null,
  preview: false,
  selectedId: null,

  name: "",
  description: "",
  publicationId: null,

  visuals: true,
  tool: "translate",

  treeIds: [],
  openIds: [],
  draggingId: null,
}));
