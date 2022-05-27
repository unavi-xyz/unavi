import produce from "immer";
import { nanoid } from "nanoid";
import { GetState, SetState } from "zustand";

import { EMPTY_SCENE } from "./constants";
import { findEntityById } from "./helpers";
import { Asset, Entity, Material, Scene } from "./types";

export type StoreSlice<T extends object, E extends object = T> = (
  set: SetState<E extends T ? E : E & T>,
  get: GetState<E extends T ? E : E & T>
) => T;

export interface ISceneSlice {
  scene: Scene;

  addEntity: (entity: Entity, parentId?: string) => void;
  updateEntity: (id: string, changes: Partial<Entity>) => void;
  removeEntity: (id: string) => void;
  moveEntity: (id: string, parentId: string, index?: number) => void;
  setMaterial: (id: string, materialId: string | undefined) => void;

  addMaterial: (material: Material) => void;
  updateMaterial: (id: string, changes: Partial<Material>) => void;
  removeMaterial: (id: string) => void;

  addAsset: (asset: Asset) => string;
  updateAsset: (id: string, changes: Partial<Asset>) => void;
  removeAsset: (id: string) => void;
}

export const createSceneSlice: StoreSlice<ISceneSlice> = (set, get) => ({
  scene: EMPTY_SCENE,

  addEntity(entity: Entity) {
    const { scene, updateEntity } = get();

    if (!entity.parentId) entity.parentId = "root";

    const parent = findEntityById(scene.tree, entity.parentId);
    if (!parent) return;

    updateEntity(parent.id, { children: [...parent.children, entity] });
  },

  updateEntity(id: string, changes: Partial<Entity>) {
    const scene = produce(get().scene, (draft) => {
      const found = findEntityById(draft.tree, id);
      if (found) Object.assign(found, changes);
    });

    set({ scene });
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
        else if (currentIndex < index)
          parent.children.splice(index - 1, 0, entity);
        else parent.children.splice(index, 0, entity);
        return;
      }

      if (entity?.parentId) {
        //remove from old parent
        const oldParent = findEntityById(draft.tree, entity.parentId);
        if (oldParent) {
          oldParent.children = oldParent.children.filter(
            (child) => child.id !== id
          );
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

  setMaterial(id: string, materialId: string | undefined) {
    const scene = produce(get().scene, (draft) => {
      const entity = findEntityById(draft.tree, id);
      if (!entity) return;

      entity.modules.forEach((module) => {
        if (module.type === "Mesh") {
          module.materialId = materialId;
        }
      });
    });

    set({ scene });
  },

  addMaterial(material: Material) {
    const scene = produce(get().scene, (draft) => {
      if (!draft.materials) draft.materials = {};
      draft.materials[material.id] = material;
    });

    set({ scene });
  },

  updateMaterial(id: string, changes: Partial<Material>) {
    const scene = produce(get().scene, (draft) => {
      const material = draft.materials[id];
      if (!material) return;

      Object.assign(material, changes);
    });

    set({ scene });
  },

  removeMaterial(id: string) {
    const scene = produce(get().scene, (draft) => {
      delete draft.materials[id];
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

  updateAsset(id: string, changes: Partial<Asset>) {
    const scene = produce(get().scene, (draft) => {
      const asset = draft.assets[id];
      if (!asset) return;

      Object.assign(asset, changes);
    });

    set({ scene });
  },

  removeAsset(id: string) {
    const scene = produce(get().scene, (draft) => {
      delete draft.assets[id];
    });

    set({ scene });
  },
});
