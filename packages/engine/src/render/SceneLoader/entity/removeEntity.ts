import { SceneMap } from "../types";
import { getChildren } from "../utils/getChildren";
import { removeEntityObject } from "./removeEntityObject";

export function removeEntity(entityId: string, map: SceneMap) {
  // Repeat for children
  const children = getChildren(entityId, map);
  children.forEach((child) => removeEntity(child.id, map));

  // Remove entity
  map.entities.delete(entityId);

  // Remove object
  removeEntityObject(entityId, map);
}
