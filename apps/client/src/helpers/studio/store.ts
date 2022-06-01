import produce from "immer";
import { WritableDraft } from "immer/dist/internal";
import { nanoid } from "nanoid";
import { MutableRefObject } from "react";
import { Group } from "three";
import create from "zustand";

import {
  Asset,
  AssetType,
  Entity,
  ISceneSlice,
  Material,
  createSceneSlice,
  findEntityById,
} from "@wired-xr/scene";

import { findFilePath, getFileByPath } from "./filesystem";
import { ENTITY_PRESETS } from "./presets";
import { Tool } from "./types";

export interface IStudioStore extends ISceneSlice {
  rootHandle: FileSystemDirectoryHandle | undefined;
  selectedDirectory: FileSystemDirectoryHandle | undefined;

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

  createAssetFromFile: (
    fileHandle: FileSystemFileHandle,
    type: AssetType
  ) => Promise<string | void>;

  setMaterialFromFile: (
    entityId: string,
    fileHandle: FileSystemFileHandle
  ) => Promise<void>;

  updateMaterialFile: (
    assetId: string,
    callback: (draft: WritableDraft<Material>) => void
  ) => Promise<void>;

  getAssetFileName: (assetId: string) => Promise<string | undefined>;
  loadAsset: (assetId: string, force?: boolean) => Promise<void>;

  addPreset: (preset: string, parentId?: string) => Entity;
}

export const useStudioStore = create<IStudioStore>((set, get) => ({
  rootHandle: undefined,
  selectedDirectory: undefined,

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

    get().addEntity(entity, parentId);

    return entity;
  },

  async createAssetFromFile(fileHandle: FileSystemFileHandle, type: AssetType) {
    //get path to file
    const root = get().rootHandle;
    if (!root) throw new Error("No root directory");
    const path = await findFilePath(root, fileHandle);
    if (!path) throw new Error("File not found");

    //see if an asset already exists
    const foundAsset = Object.entries(get().scene.assets).find(([, value]) => {
      return value.uri === path;
    });

    //if not, create one
    const assetId = foundAsset
      ? foundAsset[0]
      : get().addAsset({ type, uri: path });

    //load asset
    await get().loadAsset(assetId);

    return assetId;
  },

  async setMaterialFromFile(
    entityId: string,
    fileHandle: FileSystemFileHandle
  ) {
    //create asset
    const assetId = await get().createAssetFromFile(fileHandle, "material");
    if (!assetId) throw new Error("Failed to create asset");

    //set material
    await get().updateEntity(entityId, (draft) => {
      //@ts-ignore
      draft.props.materialId = assetId;
    });
  },

  async updateMaterialFile(
    assetId: string,
    callback: (draft: WritableDraft<Material>) => void
  ) {
    //get asset
    const asset = get().scene.assets[assetId];
    if (!asset) throw new Error("Asset not found");

    //get file
    const root = get().rootHandle;
    if (!root) throw new Error("No root directory");
    const path = asset.uri;
    const fileHandle = await getFileByPath(path, root);
    if (!fileHandle) throw new Error("File not found");

    //read file
    const file = await fileHandle.getFile();
    if (!file) throw new Error("Failed to read file");

    const text = await file.text();
    const json = JSON.parse(text) as Material;

    //update json
    const newJson = produce(json, callback);

    //if no changes, return
    if (newJson === json) return;

    //write file
    const writableStream = await fileHandle.createWritable();
    await writableStream.write(JSON.stringify(newJson, null, 2));
    await writableStream.close();
  },

  async getAssetFileName(assetId: string) {
    //get asset
    const asset = get().scene.assets[assetId];
    if (!asset) throw new Error("Asset not found");

    //get file
    const root = get().rootHandle;
    if (!root) throw new Error("No root directory");
    const path = asset.uri;
    const fileHandle = await getFileByPath(path, root);
    if (!fileHandle) throw new Error("File not found");

    //get file name
    const file = await fileHandle.getFile();
    if (!file) throw new Error("Failed to read file");

    return file.name;
  },

  async loadAsset(assetId: string, force = false) {
    const newScene = await produce(get().scene, async (draft) => {
      //get asset
      const asset = draft.assets[assetId];
      if (!asset) throw new Error("Asset not found");

      //if already loaded, return
      //unless force is true
      if (asset.data && !force) return;

      //get file
      const root = get().rootHandle;
      if (!root) throw new Error("No root directory");
      const fileHandle = await getFileByPath(asset.uri, root);
      if (!fileHandle) throw new Error("File not found");
      const file = await fileHandle.getFile();
      if (!file) throw new Error("Failed to read file");

      if (asset.type === "image") {
        const url = URL.createObjectURL(file);
        asset.data = url;
      } else if (asset.type === "material") {
        const text = await file.text();
        const json = JSON.parse(text);

        draft.assets[assetId].data = json;
      }
    });

    set({ scene: newScene });
  },

  ...createSceneSlice(set as any, get),
}));
