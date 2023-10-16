import { SyncedMesh, syncedStore } from "../store";

export function editMesh(id: string, data: Omit<Partial<SyncedMesh>, "id">) {
  const mesh = syncedStore.meshes[id];
  if (!mesh) return;

  Object.assign(mesh, data);
}
