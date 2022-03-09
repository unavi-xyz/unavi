import { atom } from "jotai";
import { Euler, Quaternion, Vector3 } from "three";
import { Scene } from "3d";

import { Selected, Tool } from "./types";

const tempVector3 = new Vector3();
const tempQuaternion = new Quaternion();
const tempEuler = new Euler();

export const sceneAtom = atom(null as Scene);
export const worldIdAtom = atom(null as string);

export const toolAtom = atom(Tool.translate);

export const usingGizmoAtom = atom(false);
export const selectedAtom = atom(null as Selected);

export const saveSelectedAtom = atom(
  (get) => get(selectedAtom),
  (get, set) => {
    const newSelected = { ...get(selectedAtom) };
    const { instance, ref } = newSelected;

    const position = ref.current.getWorldPosition(tempVector3).toArray();
    instance.params.position = position;

    const rotationQuat = ref.current.getWorldQuaternion(tempQuaternion);
    const rotationEuler = tempEuler.setFromQuaternion(rotationQuat);
    const rotation = tempVector3.setFromEuler(rotationEuler).toArray();
    instance.params.rotation = rotation;

    const scale = ref.current.getWorldScale(tempVector3).toArray();
    instance.params.scale = scale;

    set(selectedAtom, newSelected);
  }
);
