import { SyncedNode, syncedStore } from "../store";

export function editNode(id: string, data: Omit<Partial<SyncedNode>, "id">) {
  const node = syncedStore.nodes[id];
  if (!node) return;

  Object.assign(node, data);
}
