import { Entity } from "./types";

export function findEntityById(entity: Entity, id: string): Entity | undefined {
  if (entity.id === id) return entity;

  for (const child of entity.children) {
    const found = findEntityById(child, id);
    if (found) return found;
  }

  return undefined;
}

export function traverseTree(
  entity: Entity,
  callback: (entity: Entity) => void
) {
  callback(entity);

  for (const child of entity.children) {
    traverseTree(child, callback);
  }
}
