import { deleteId } from "../entities";
import { syncedStore } from "../store";

export function removeMesh(id: string) {
  delete syncedStore.meshes[id];
  deleteId(id);
}
