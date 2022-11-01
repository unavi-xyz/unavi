import { Mesh } from "three";

import { disposeMaterial } from "../../utils/disposeObject";
import { defaultMaterial } from "../constants";
import { SceneMap } from "../types";

export function removeMaterial(materialId: string, map: SceneMap) {
  const material = map.materials.get(materialId);
  if (!material) throw new Error(`Material not found: ${materialId}`);

  // Remove material from meshes
  map.meshes.forEach((mesh) => {
    if (mesh.materialId === materialId) {
      mesh.materialId = null;

      // Remove from object
      const object = map.objects.get(mesh.id);
      if (object instanceof Mesh) {
        object.material = defaultMaterial;
      }
    }
  });

  // Remove from materials
  map.materials.delete(materialId);

  // Dispose material
  disposeMaterial(material);
}
