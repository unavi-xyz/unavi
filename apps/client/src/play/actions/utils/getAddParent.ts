import { editorStore, rootNameAtom } from "@unavi/engine";
import { getDefaultStore } from "jotai";

export function getAddParent() {
  const sceneTreeId = editorStore.get(editorStore.sceneTreeId);
  const items = editorStore.get(editorStore.items);
  const rootName = getDefaultStore().get(rootNameAtom);

  if (sceneTreeId) {
    const item = items.get(sceneTreeId);

    if (item) {
      return item.name;
    }
  }

  return rootName;
}
