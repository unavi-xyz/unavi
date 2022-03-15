import create, { SetState, GetState } from "zustand";
import { nanoid } from "nanoid";
import { Euler, Quaternion, Vector3 } from "three";
import { AssetName, ASSETS, Params, Scene } from "3d";

import { Selected } from "./types";

type StoreState = {
  scene: Scene;
  setScene: (value: Scene) => void;

  selected: Selected;
  setSelected: (value: Selected) => void;
  saveSelected: () => void;

  updateInstanceParams: (id: string, changes: Partial<Params>) => void;
  newInstance: (name: AssetName, initialParams?: Partial<Params>) => void;
  deleteInstance: (id: string) => void;
};

export const useStore = create<StoreState>(
  (set: SetState<StoreState>, get: GetState<StoreState>) => ({
    scene: {},
    setScene: (value: Scene) => set({ scene: value }),

    selected: null,
    setSelected: (value: Selected) => set({ selected: value }),
    saveSelected: () => {
      const { id, ref } = get().selected;
      const params = get().scene[id].params;

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
      const params = get().scene[id].params;
      const newParams = { ...params, ...changes };

      const scene = get().scene;
      scene[id].params = newParams;

      set({ scene });
    },

    newInstance: (name: AssetName, initialParams?: Partial<Params>) => {
      const id = nanoid();
      const params = initialParams ?? { ...ASSETS[name].params };

      const newScene = { ...get().scene };
      newScene[id] = { id, name, params };

      get().setScene(newScene);
    },

    deleteInstance(id: string) {
      const newScene = { ...get().scene };
      delete newScene[id];

      get().setScene(newScene);
    },
  })
);
