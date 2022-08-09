import { Group, Object3D } from "three";
import create from "zustand";

import { Engine } from "@wired-xr/new-engine";

import { Tool } from "./types";

export interface IStudioStore {
  engine: Engine | null;
  canvas: HTMLCanvasElement | null;
  root: Object3D;

  name: string;
  description: string;

  preview: boolean;
  debug: boolean;
  grid: boolean;
  tool: Tool;
  selectedId: string | null;

  usingTransform: boolean;
  treeNonce: number;
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  engine: null,
  canvas: null,
  root: new Group(),

  name: "",
  description: "",

  preview: false,
  debug: false,
  grid: false,

  tool: "translate",
  usingTransform: false,
  selectedId: null,

  treeNonce: 0,
}));
