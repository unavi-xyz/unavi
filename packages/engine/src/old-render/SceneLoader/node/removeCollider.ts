import { disposeObject } from "../../utils/disposeObject";
import { SceneMap } from "../types";

export function removeCollider(nodeId: string, map: SceneMap) {
  const collider = map.colliders.get(nodeId);
  if (!collider) return;

  map.colliders.delete(nodeId);

  disposeObject(collider);
}
