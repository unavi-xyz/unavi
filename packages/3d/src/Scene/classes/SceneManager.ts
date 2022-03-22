import { UseBoundStore, StoreApi } from "zustand";
import { nanoid } from "nanoid";

import { SceneObjects } from "../constants";
import {
  Instance,
  Material,
  Properties,
  Scene,
  SceneObjectType,
} from "../types";
import { fileToArrayBuffer } from "../helpers";
import { defaultMaterial } from "../modules/MeshMaterial";

const Hash = require("ipfs-only-hash");

export type SceneStore = {
  scene: Scene;
  [key: string]: unknown;
};

type useStoreType = UseBoundStore<SceneStore, StoreApi<SceneStore>>;

export class SceneManager {
  useStore: useStoreType;

  constructor(useStore: useStoreType) {
    this.useStore = useStore;
  }

  newInstance(type: SceneObjectType) {
    const id = nanoid();
    const properties = SceneObjects[type].properties;
    const instance: Instance<typeof type> = { id, type, properties };

    const scene = this.useStore.getState().scene;
    const newInstances = { ...scene.instances };
    newInstances[id] = instance;
    scene.instances = newInstances;

    this.useStore.setState({ scene });
    return id;
  }

  editInstance(id: string, properties: Partial<Properties>) {
    const scene = this.useStore.getState().scene;
    const instance = scene.instances[id];
    instance.properties = { ...instance.properties, ...properties };

    this.useStore.setState({ scene });
  }

  deleteInstance(id: string) {
    const newScene = { ...this.useStore.getState().scene };
    delete newScene.instances[id];
    this.useStore.setState({ scene: newScene });
  }

  async newAsset(file: File) {
    //use a hash as the key for each asset
    //this ensures no duplicate assets, while allowing for easy IPFS integration
    const array = new Uint8Array(await fileToArrayBuffer(file));
    const cid: string = await Hash.of(array);

    const scene = this.useStore.getState().scene;
    const newAssets = { ...scene.assets };
    newAssets[cid] = file;
    scene.assets = newAssets;

    this.useStore.setState({ scene });
    return cid;
  }

  deleteAsset(id: string) {
    const scene = this.useStore.getState().scene;
    const newAssets = { ...scene.assets };
    delete newAssets[id];
    scene.assets = newAssets;

    this.useStore.setState({ scene });
  }

  pruneAssets() {
    //delete unused assets
    const scene = this.useStore.getState().scene;

    const usedTextures = Object.values(scene.materials).map((material) => {
      return material?.texture;
    });

    const usedSrc = Object.values(scene.instances).map(({ properties }) => {
      if ("src" in properties) return properties.src;
    });

    const usedAssets = Object.keys(scene.assets).filter((id) => {
      return usedTextures.includes(id) || usedSrc.includes(id);
    });

    const newAssets: typeof scene.assets = {};
    usedAssets.forEach((id) => {
      newAssets[id] = scene.assets[id];
    });
    scene.assets = newAssets;
  }

  newMaterial() {
    const id = nanoid();
    const material = { ...defaultMaterial, id };

    const scene = this.useStore.getState().scene;
    const newMaterials = { ...scene.materials };
    newMaterials[id] = material;

    scene.materials = newMaterials;
    this.useStore.setState({ scene });

    return id;
  }

  deleteMaterial(id: string) {
    //remove from all scene objects
    const scene = this.useStore.getState().scene;
    Object.values(scene.instances).forEach((object) => {
      if ("material" in object.properties) {
        if (object.properties.material === id) {
          object.properties.material = undefined;
        }
      }
    });

    //delete
    const newMaterials = { ...scene.materials };
    delete newMaterials[id];
    scene.materials = newMaterials;

    this.useStore.setState({ scene });
  }

  editMaterial(id: string, changes: Partial<Material>) {
    const scene = this.useStore.getState().scene;
    const newMaterials = { ...scene.materials };
    const newMaterial = { ...newMaterials[id], ...changes };
    newMaterials[id] = newMaterial;
    scene.materials = newMaterials;

    this.useStore.setState({ scene });
  }

  toJSON() {
    const scene = this.useStore.getState().scene;
    const json = JSON.stringify(scene);
    return json;
  }

  fromJSON(json: string) {
    const scene = JSON.parse(json) as Scene;
    this.useStore.setState({ scene });
  }

  get scene() {
    return this.useStore.getState().scene;
  }

  set scene(scene: Scene) {
    this.useStore.setState({ scene });
  }
}
