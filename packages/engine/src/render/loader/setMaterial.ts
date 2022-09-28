import { Mesh } from "three";

import { defaultMaterial } from "./constants";
import { SceneMap } from "./SceneMap";

export function setMaterial(
  entityId: string,
  materialId: string | null,
  map: SceneMap
) {
  const object = map.objects.get(entityId);
  if (!object) throw new Error("Object not found");
  const isMesh = object instanceof Mesh;
  if (!isMesh) {
    if (materialId === null) return;
    throw new Error("Object is not a mesh");
  }

  const material = materialId ? map.materials.get(materialId) : defaultMaterial;
  if (!material) throw new Error("Material not found");

  // Set material
  object.material = material;
}
