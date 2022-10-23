import { disposeObject } from "../../utils/disposeObject";
import { SceneMap } from "../types";
import { removeColliderVisual } from "./removeColliderVisual";

export function removeEntityObject(entityId: string, map: SceneMap) {
  // Don't remove root object
  if (entityId === "root") return;

  // Remove collider visual
  removeColliderVisual(entityId, map);

  // Remove object
  const object = map.objects.get(entityId);
  if (!object) return;

  map.objects.delete(entityId);

  // Dispose object
  disposeObject(object);
}
