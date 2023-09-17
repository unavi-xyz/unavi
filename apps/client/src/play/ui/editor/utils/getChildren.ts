import { SyncedNode, SyncedStore, syncedStore } from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";

export function getChildren(
  id: string,
  store: DeepReadonly<SyncedStore> = syncedStore
): DeepReadonly<SyncedNode>[] {
  const node = store.nodes[id];

  if (node) {
    return Object.values(store.nodes).filter((n) => n.parentId === id);
  }

  const scene = store.scenes[id];

  if (scene) {
    return scene.nodeIds
      .map((id) => store.nodes[id])
      .filter((n): n is SyncedNode => n !== undefined);
  }

  return [];
}
