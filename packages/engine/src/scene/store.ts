import produce from "immer";
import { WritableDraft } from "immer/dist/internal";
import { nanoid } from "nanoid";
import { GetState, SetState } from "zustand";

import { EMPTY_SCENE } from "./constants";
import { Asset, IEntity, IScene } from "./types";
import { findEntityById } from "./utils";

export type StoreSlice<T extends object, E extends object = T> = (
  set: SetState<E extends T ? E : E & T>,
  get: GetState<E extends T ? E : E & T>
) => T;

export interface ISceneSlice {
  scene: IScene;

  addEntity: (entity: IEntity, parentId?: string) => void;
  removeEntity: (id: string) => void;
  moveEntity: (id: string, parentId: string, index?: number) => void;

  addAsset: (asset: Asset) => string;
  removeAsset: (id: string) => void;

  updateEntity: (id: string, callback: (draft: WritableDraft<IEntity>) => void) => void;
  updateAsset: (id: string, callback: (draft: WritableDraft<Asset>) => void) => void;
}

export const createSceneSlice: StoreSlice<ISceneSlice> = (set, get) => ({
  scene: EMPTY_SCENE,

  addEntity(entity: IEntity) {
    const { scene, updateEntity } = get();

    if (!entity.parentId) entity.parentId = "root";

    const parent = findEntityById(scene.tree, entity.parentId);
    if (!parent) return;

    updateEntity(parent.id, (draft) => {
      draft.children.push(entity);
    });
  },

  removeEntity(id: string) {
    const scene = produce(get().scene, (draft) => {
      const entity = findEntityById(draft.tree, id);
      if (!entity?.parentId) return;

      const parent = findEntityById(draft.tree, entity.parentId);
      if (!parent) return;

      //remove from parent
      const index = parent.children.indexOf(entity);
      if (index !== -1) parent.children.splice(index, 1);
    });

    set({ scene });
  },

  moveEntity(id: string, parentId: string, index: number = -1) {
    const scene = produce(get().scene, (draft) => {
      const entity = findEntityById(draft.tree, id);
      if (!entity) return;

      const isInParent = entity.parentId === parentId;

      //if already in parent, move to index
      if (isInParent) {
        const parent = findEntityById(draft.tree, parentId);
        if (!parent) return;
        const currentIndex = parent.children.indexOf(entity);

        parent.children = parent.children.filter((child) => child.id !== id);

        if (index === -1) parent.children.push(entity);
        else if (currentIndex < index) parent.children.splice(index - 1, 0, entity);
        else parent.children.splice(index, 0, entity);
        return;
      }

      if (entity?.parentId) {
        //remove from old parent
        const oldParent = findEntityById(draft.tree, entity.parentId);
        if (oldParent) {
          oldParent.children = oldParent.children.filter((child) => child.id !== id);
        }
      }

      //get new parent
      const parent = findEntityById(draft.tree, parentId);
      if (!parent) return;

      //add to new parent
      entity.parentId = parentId;

      //add to parent children
      if (index === -1) parent.children.push(entity);
      else parent.children.splice(index, 0, entity);
    });

    set({ scene });
  },

  addAsset(asset: Asset) {
    const id = nanoid();

    const scene = produce(get().scene, (draft) => {
      if (!draft.assets) draft.assets = {};
      draft.assets[id] = asset;
    });

    set({ scene });

    return id;
  },

  removeAsset(id: string) {
    const scene = produce(get().scene, (draft) => {
      delete draft.assets[id];
    });

    set({ scene });
  },

  updateEntity(id: string, callback: (draft: WritableDraft<IEntity>) => void) {
    const scene = produce(get().scene, (draft) => {
      const found = findEntityById(draft.tree, id);
      if (found) callback(found);
    });

    set({ scene });
  },

  updateAsset(id: string, callback: (draft: WritableDraft<Asset>) => void) {
    const scene = produce(get().scene, (draft) => {
      const asset = draft.assets[id];
      if (asset) callback(asset);
    });

    set({ scene });
  },
});
