import { deleteId } from "../entities";
import { syncedStore } from "../store";

export function removeNode(id: string) {
  delete syncedStore.nodes[id];
  deleteId(id);
}
