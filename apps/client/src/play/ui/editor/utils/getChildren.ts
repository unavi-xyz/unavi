import { SyncedNode, SyncedStore, syncedStore } from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";

export function getChildren(
  id: string,
  store: DeepReadonly<SyncedStore> = syncedStore
): DeepReadonly<SyncedNode>[] {
  return Object.values(store.nodes).filter((n) => n.parentId === id);
}
