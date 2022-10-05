import { Engine, Entity } from "@wired-labs/engine";
import create from "zustand";

import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;

  getEntity: (id: string) => Entity | undefined;

  exportedScene: Uint8Array | null;

  canvas: HTMLCanvasElement | null;
  preview: boolean;
  selectedId: string | null;

  name: string;
  description: string;

  colliders: boolean;
  grid: boolean;
  tool: Tool;
}

export const useEditorStore = create<IEditorStore>((set, get) => ({
  engine: null,

  getEntity: (id: string) => {
    return get().engine?.scene.entities[id];
  },

  exportedScene: null,

  canvas: null,
  preview: false,
  selectedId: null,

  name: "",
  description: "",

  colliders: true,
  grid: false,
  tool: "translate",
}));
