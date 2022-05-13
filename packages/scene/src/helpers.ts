import { Vector3 } from "three";

import { Entity } from "./types";

const tempVector3 = new Vector3();

export function findEntityById(entity: Entity, id: string): Entity | undefined {
  if (entity.id === id) return entity;

  for (const child of entity.children) {
    const found = findEntityById(child, id);
    if (found) return found;
  }

  return undefined;
}

// export function getWorldTransform(entity: Entity, scene:Entity):Transform {

//   if (!entity.parentId) return entity.transform;

//   const parent = findEntityById(scene, entity.parentId);

// }
