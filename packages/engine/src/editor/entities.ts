import { syncedStore } from "./store";

const entityMap = new Map<string, bigint>();
const entityMapReverse = new Map<bigint, string>();

export function getEntityId(id: string) {
  return entityMap.get(id);
}

export function getId(entityId: bigint) {
  return entityMapReverse.get(entityId);
}

export function setEntityId(id: string, entityId: bigint) {
  entityMap.set(id, entityId);
  entityMapReverse.set(entityId, id);
}

export function deleteId(id: string) {
  const entityId = entityMap.get(id);
  if (!entityId) return;

  entityMap.delete(id);
  entityMapReverse.delete(entityId);
}

export function clearEntityIds() {
  entityMap.clear();
  entityMapReverse.clear();
}

export function getSceneByEntityId(entityId: bigint) {
  const id = getId(entityId);
  if (!id) return undefined;
  return syncedStore.scenes[id];
}

export function getNodeByEntityId(entityId: bigint) {
  const id = getId(entityId);
  if (!id) return undefined;
  return syncedStore.nodes[id];
}

export function getMeshByEntityId(entityId: bigint) {
  const id = getId(entityId);
  if (!id) return undefined;
  return syncedStore.meshes[id];
}
