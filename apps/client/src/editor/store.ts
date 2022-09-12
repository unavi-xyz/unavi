import create from "zustand";

import { Engine, Scene } from "@wired-labs/engine";

import { deepClone } from "../utils/deepClone";
import { emptyScene } from "./constants";
import { Tool } from "./types";

export interface IEditorStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;
  preview: boolean;
  selectedId: string | null;

  name: string;
  description: string;
  image: string;

  scene: Scene;

  debug: boolean;
  grid: boolean;
  tool: Tool;
}

export const useEditorStore = create<IEditorStore>(() => ({
  engine: null,
  canvas: null,
  preview: false,
  selectedId: null,

  name: "",
  description: "",
  image: "",

  scene: deepClone(emptyScene),

  debug: false,
  grid: false,
  tool: "translate",
}));
