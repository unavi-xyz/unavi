import create from "zustand";

import { Engine, Entity } from "@wired-xr/engine";

import { Tool } from "./types";

export interface IStudioStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;
  name: string;
  description: string;
  preview: boolean;
  selectedId: string | null;

  treeNonce: number;
  tree: { [id: string]: Entity };

  debug: boolean;
  grid: boolean;
  tool: Tool;
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  engine: null,
  canvas: null,
  name: "",
  description: "",
  preview: false,
  selectedId: null,

  treeNonce: 0,
  tree: {},

  debug: false,
  grid: false,
  tool: "translate",
}));
