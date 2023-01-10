import { disposeObject } from "../../utils/disposeObject";
import { SceneMap } from "../types";

export function removeMesh(meshId: string, map: SceneMap) {
  // Remove from map
  map.meshes.delete(meshId);

  // Remove object
  const object = map.objects.get(meshId);
  if (!object) return;

  map.objects.delete(meshId);

  disposeObject(object);
}
