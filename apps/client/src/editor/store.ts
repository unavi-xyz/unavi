import { Engine, Entity } from "@wired-labs/engine";
import create from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;
  sceneLoaded: boolean;

  getEntity: (id: string) => Entity | undefined;

  canvas: HTMLCanvasElement | null;
  preview: boolean;
  selectedId: string | null;

  name: string;
  description: string;

  colliders: boolean;
  tool: Tool;
}

export const useEditorStore = create<IEditorStore>((set, get) => ({
  engine: null,
  sceneLoaded: false,

  getEntity: (id: string) => {
    return get().engine?.scene.entities[id];
  },

  canvas: null,
  preview: false,
  selectedId: null,

  name: "",
  description: "",

  colliders: false,
  tool: "translate",
}));
