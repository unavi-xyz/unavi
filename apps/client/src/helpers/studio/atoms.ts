import { atom } from "jotai";
import { atomWithStore } from "jotai/zustand";

import { findEntityById } from "@wired-xr/scene";

import { useStudioStore } from "./store";

const studioStoreAtom = atomWithStore(useStudioStore);

export const selectedAtom = atom((get) => {
  const tree = get(studioStoreAtom).scene.tree;
  const id = get(studioStoreAtom).selectedId;
  if (!id) return undefined;

  const object = findEntityById(tree, id);
  return object;
});

export const selectedRefAtom = atom((get) => {
  const treeRefs = get(studioStoreAtom).treeRefs;
  const id = get(studioStoreAtom).selectedId;
  if (!id) return undefined;

  const ref = treeRefs[id];
  return ref;
});
