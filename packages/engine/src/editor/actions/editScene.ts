import { SyncedScene, syncedStore } from "../store";

export function editScene(id: string, data: Omit<Partial<SyncedScene>, "id">) {
  const scene = syncedStore.scenes[id];
  if (!scene) return;

  Object.assign(scene, data);
}
