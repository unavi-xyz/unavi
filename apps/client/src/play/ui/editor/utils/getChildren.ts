import { SyncedNode, SyncedStore, syncedStore } from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";

export function getChildren(
  id: string,
  store: DeepReadonly<SyncedStore> = syncedStore
): DeepReadonly<SyncedNode>[] {
  const children: DeepReadonly<SyncedNode>[] = [];

  for (const nodeId in store.nodes) {
    const node = store.nodes[nodeId];

    if (node?.parentId === id) {
      children.push(node);
    }
  }

  return children;
}
