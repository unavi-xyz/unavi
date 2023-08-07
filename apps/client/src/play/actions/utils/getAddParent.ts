import { useClientStore, useSceneStore } from "@unavi/engine";

export function getAddParent() {
  const { sceneTreeId, items } = useSceneStore.getState();

  if (sceneTreeId) {
    const item = items.get(sceneTreeId);

    if (item) {
      return item.name;
    }
  }

  const { rootName } = useClientStore.getState();

  return rootName;
}
