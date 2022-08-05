import { Group, Object3D } from "three";
import create from "zustand";

import { Engine } from "@wired-xr/new-engine";

import { Tool } from "./types";

export interface IStudioStore {
  engine: Engine | null;
  root: Object3D;

  name: string | undefined;
  description: string | undefined;

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
  root: new Group(),

  name: undefined,
  description: undefined,

  preview: false,
  debug: false,
  grid: false,

  tool: "translate",
  usingTransform: false,
  selectedId: null,

  treeNonce: 0,
}));
