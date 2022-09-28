import { Entity } from "../../scene";
import { removeEntityObject } from "./removeEntityObject";
import { SceneMap } from "./SceneMap";

export function removeEntity(entity: Entity, map: SceneMap) {
  // Repeat for children
  entity.children.forEach((child) => {
    if (!child) throw new Error("No child");
    removeEntity(child, map);
  });

  // Remove object
  removeEntityObject(entity.id, map);
}
