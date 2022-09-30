import { EntityJSON } from "../../../scene";
import { SceneMap } from "../types";

export function getChildren(entityId: string, map: SceneMap) {
  const children: EntityJSON[] = [];

  map.entities.forEach((e) => {
    if (e.parentId === entityId) children.push(e);
  });

  return children;
}
