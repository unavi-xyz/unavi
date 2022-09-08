import create from "zustand";

import { Engine, Entity } from "@wired-labs/engine";

import { deepClone } from "../utils/deepClone";
import { emptyTree } from "./constants";
import { Tool } from "./types";

export interface IEditorStore {
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

export const useEditorStore = create<IEditorStore>(() => ({
  engine: null,
  canvas: null,
  name: "",
  description: "",
  preview: false,
  selectedId: null,

  treeNonce: 0,
  tree: deepClone(emptyTree),

  debug: false,
  grid: false,
  tool: "translate",
}));
