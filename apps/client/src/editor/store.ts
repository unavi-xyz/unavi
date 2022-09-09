import create from "zustand";

import { Engine, Scene } from "@wired-labs/engine";

import { deepClone } from "../utils/deepClone";
import { emptyScene } from "./constants";
import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;
  name: string;
  description: string;
  preview: boolean;
  selectedId: string | null;

  scene: Scene;

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

  scene: deepClone(emptyScene),

  debug: false,
  grid: false,
  tool: "translate",
}));
