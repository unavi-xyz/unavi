import create, { SetState, GetState } from "zustand";
import { nanoid } from "nanoid";
import { Euler, Quaternion, Vector3 } from "three";
import { AssetName, ASSETS, Params, Scene } from "3d";

import { Selected } from "./types";
import {
  readFileAsArrayBuffer,
  readFileAsDataUrl,
} from "../../../helpers/files";

const Hash = require("ipfs-only-hash");

type StoreState = {
  scene: Scene;
  setScene: (value: Scene) => void;

  selected: Selected;
  setSelected: (value: Selected) => void;
  saveSelected: () => void;

  updateInstanceParams: (id: string, changes: Partial<Params>) => void;
  newInstance: (name: AssetName, initialParams?: Partial<Params>) => void;
  deleteInstance: (id: string) => void;

  newTexture: (file: File) => Promise<string>;
  newGLTF: (file: File) => Promise<string>;
};

export const useStore = create<StoreState>(
  (set: SetState<StoreState>, get: GetState<StoreState>) => ({
    scene: null,
    setScene: (value: Scene) => set({ scene: value }),

    selected: null,
    setSelected: (value: Selected) => set({ selected: value }),
    saveSelected: () => {
      const { id, ref } = get().selected;
      const params = get().scene.instances[id].params;

      const position = ref.current.getWorldPosition(new Vector3()).toArray();

      const rotationQuat = ref.current.getWorldQuaternion(new Quaternion());
      const rotationEuler = new Euler().setFromQuaternion(rotationQuat);
      const rotation = new Vector3().setFromEuler(rotationEuler).toArray();

      const changes: typeof params = { position, rotation };

      if (Boolean(params?.scale)) {
        changes.scale = ref.current.getWorldScale(new Vector3()).toArray();
      }

      get().updateInstanceParams(id, changes);
    },

    updateInstanceParams: (id: string, changes: Partial<Params>) => {
      const params = get().scene.instances[id].params;
      const newParams = { ...params, ...changes };

      const scene = get().scene;
      scene.instances[id].params = newParams;

      set({ scene });
    },

    newInstance: (name: AssetName, initialParams?: Partial<Params>) => {
      const id = nanoid();
      const params = initialParams ?? { ...ASSETS[name].params };

      const newScene = { ...get().scene };
      newScene.instances[id] = { id, name, params };

      set({ scene: newScene });
    },

    deleteInstance(id: string) {
      const newScene = { ...get().scene };
      delete newScene.instances[id];

      set({ scene: newScene });
    },

    async newTexture(file: File) {
      const array = new Uint8Array(await readFileAsArrayBuffer(file));
      const cid = await Hash.of(array);

      const value = await readFileAsDataUrl(file);

      const newScene = { ...get().scene };
      newScene.textures[cid] = { value, name: file.name };

      set({ scene: newScene });

      return cid;
    },

    async newGLTF(file: File) {
      const array = new Uint8Array(await readFileAsArrayBuffer(file));
      const cid = await Hash.of(array);

      const value = await readFileAsDataUrl(file);

      const newScene = { ...get().scene };
      newScene.models[cid] = { value, name: file.name };

      set({ scene: newScene });

      return cid;
    },
  })
);
