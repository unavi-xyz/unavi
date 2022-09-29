import { Entity } from "../../../scene";
import { SceneMap } from "../types";
import { removeEntityObject } from "./removeEntityObject";

export function removeEntity(entity: Entity, map: SceneMap) {
  // Repeat for children
  entity.children.forEach((child) => {
    if (!child) throw new Error("No child");
    removeEntity(child, map);
  });

  // Remove object
  removeEntityObject(entity.id, map);
}
