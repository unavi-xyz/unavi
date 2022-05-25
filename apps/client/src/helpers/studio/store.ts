import produce from "immer";
import { nanoid } from "nanoid";
import { MutableRefObject } from "react";
import { Group } from "three";
import create from "zustand";

import { Entity, ISceneSlice, createSceneSlice } from "@wired-xr/scene";

import { ENTITY_PRESETS } from "./presets";
import { Tool } from "./types";

export interface IStudioStore extends ISceneSlice {
  rootHandle: FileSystemDirectoryHandle | undefined;

  preview: boolean;
  debug: boolean;

  tool: Tool;
  usingGizmo: boolean;
  selectedId: string | undefined;

  closedInspectMenus: string[];
  toggleClosedInspectMenu: (name: string) => void;

  treeRefs: { [id: string]: MutableRefObject<Group | null> };
  setRef: (id: string, ref: MutableRefObject<Group | null>) => void;
  removeRef: (id: string) => void;

  addPreset: (preset: string, parentId?: string) => Entity;
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  rootHandle: undefined,

  preview: false,
  debug: false,

  tool: "translate",
  usingGizmo: false,
  selectedId: undefined,
  treeRefs: {},

  closedInspectMenus: [],

  toggleClosedInspectMenu(name: string) {
    set(
      produce(get(), (draft) => {
        const index = draft.closedInspectMenus.indexOf(name);
        if (index > -1) {
          draft.closedInspectMenus.splice(index, 1);
        } else {
          draft.closedInspectMenus.push(name);
        }
      })
    );
  },

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
