import { Entity } from "./types";

export function findEntityById(entity: Entity, id: string): Entity | undefined {
  if (entity.id === id) return entity;

  for (const child of entity.children) {
    const found = findEntityById(child, id);
    if (found) return found;
  }

  return undefined;
}
