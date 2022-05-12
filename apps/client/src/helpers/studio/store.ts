import { MutableRefObject } from "react";
import { Group } from "three";
import create, { SetState, GetState } from "zustand";
import { nanoid } from "nanoid";
import { createSceneSlice, Entity, ISceneSlice } from "scene";

import { Tool } from "./types";
import { ENTITY_PRESETS } from "./presets";

export interface IStudioStore extends ISceneSlice {
  id: string | undefined;
  name: string | undefined;
  preview: boolean;
  tool: Tool;
  usingGizmo: boolean;
  selectedId: string | undefined;

  openTreeObjects: Set<string>;
  treeRefs: { [id: string]: MutableRefObject<Group | null> };

  setRef: (id: string, ref: MutableRefObject<Group | null>) => void;
  removeRef: (id: string) => void;

  addPreset: (preset: string, parentId?: string) => Entity;
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  id: undefined,
  name: undefined,
  preview: false,
  tool: "translate",
  usingGizmo: false,
  selectedId: undefined,

  openTreeObjects: new Set(),
  treeRefs: {},

  setRef(id: string, ref: MutableRefObject<Group | null>) {
    set({ treeRefs: { ...get().treeRefs, [id]: ref } });
  },

  removeRef(id: string) {
    const newTreeRefs = { ...get().treeRefs };
    delete newTreeRefs[id];
    set({ treeRefs: newTreeRefs });
  },

  addPreset(preset: string, parentId = "root") {
    const entity = { ...ENTITY_PRESETS[preset] };
    entity.id = nanoid();
    entity.modules = entity.modules.map((module) => {
      const newModule = { ...module };
      newModule.id = nanoid();
      return newModule;
    });

    get().addEntity(entity, parentId);

    return entity;
  },

  ...createSceneSlice(set as any, get),
}));
