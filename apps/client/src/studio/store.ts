import create from "zustand";

import { Engine } from "@wired-xr/engine";

import { Tool } from "./types";

export interface IStudioStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;

  name: string;
  description: string;

  debug: boolean;
  grid: boolean;
  tool: Tool;
  selectedId: string | null;

  preview: boolean;
  usingTransformControls: boolean;
  treeNonce: number;
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  engine: null,
  canvas: null,

  name: "",
  description: "",

  preview: false,
  debug: false,
  grid: false,

  tool: "translate",
  usingTransformControls: false,
  selectedId: null,

  treeNonce: 0,
}));
