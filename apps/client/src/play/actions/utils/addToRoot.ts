import { syncedStore, useSceneStore } from "@unavi/engine";

export function addToRoot(nodeId: string) {
  const { rootId } = useSceneStore.getState();

  if (rootId) {
    const root = syncedStore.scenes[rootId];

    if (root) {
      root.nodeIds.push(nodeId);
    }
  }
}
