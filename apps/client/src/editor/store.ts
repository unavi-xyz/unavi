import { Engine, Node } from "@wired-labs/engine";
import create from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;
  sceneLoaded: boolean;

  getNode: (id: string) => Node | undefined;

  canvas: HTMLCanvasElement | null;
  preview: boolean;
  selectedId: string | null;

  name: string;
  description: string;

  visuals: boolean;
  tool: Tool;
}

export const useEditorStore = create<IEditorStore>((set, get) => ({
  engine: null,
  sceneLoaded: false,

  getNode: (id: string) => {
    return get().engine?.scene.nodes[id];
  },

  canvas: null,
  preview: false,
  selectedId: null,

  name: "",
  description: "",

  visuals: true,
  tool: "translate",
}));
