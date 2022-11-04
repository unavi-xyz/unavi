import { disposeObject } from "../../utils/disposeObject";
import { SceneMap } from "../types";
import { removeColliderVisual } from "./removeColliderVisual";

export function removeNodeObject(nodeId: string, map: SceneMap) {
  // Don't remove root object
  if (nodeId === "root") return;

  // Remove collider visual
  removeColliderVisual(nodeId, map);

  // Remove object
  const object = map.objects.get(nodeId);
  if (!object) return;

  map.objects.delete(nodeId);

  // Dispose object
  disposeObject(object);
}
