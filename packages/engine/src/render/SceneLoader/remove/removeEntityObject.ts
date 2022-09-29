import { disposeObject } from "../../../utils/disposeObject";
import { SceneMap } from "../types";
import { removeColliderVisual } from "./removeColliderVisual";

export function removeEntityObject(entityId: string, map: SceneMap) {
  const object = map.objects.get(entityId);
  if (!object) throw new Error(`Object not found: ${entityId}`);

  // Don't remove root object
  if (entityId === "root") return;

  // Remove from scene
  object.removeFromParent();
  map.objects.delete(entityId);

  // Dispose object
  disposeObject(object);

  // Remove collider visual
  removeColliderVisual(entityId, map);
}
