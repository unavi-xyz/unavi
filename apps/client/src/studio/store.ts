import create from "zustand";

import { Engine } from "@wired-xr/engine";

import { Tool } from "./types";

export interface IStudioStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;
  name: string;
  description: string;
  preview: boolean;
  selectedId: number | null;
  treeNonce: number;

  debug: boolean;
  grid: boolean;
  tool: Tool;
  names: { [id: number]: string };
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  engine: null,
  canvas: null,
  name: "",
  description: "",
  preview: false,
  selectedId: null,
  treeNonce: 0,

  debug: false,
  grid: false,
  tool: "translate",
  names: {},
}));
