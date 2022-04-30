import create, { SetState, GetState } from "zustand";
import { createSceneSlice, ISceneSlice, Primitive, TreeObject } from "scene";

import { Tool } from "./types";

export interface IStudioStore extends ISceneSlice {
  tool: Tool;
  selected: TreeObject<Primitive> | undefined;
}

export const useStudioStore = create<IStudioStore>(
  (set: SetState<IStudioStore>, get: GetState<IStudioStore>) => ({
    tool: "Translate",
    selected: undefined,

    ...createSceneSlice(set as any, get),
  })
);
