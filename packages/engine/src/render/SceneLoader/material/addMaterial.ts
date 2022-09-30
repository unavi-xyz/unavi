import { MeshStandardMaterial } from "three";

import { MaterialJSON } from "../../../scene";
import { SceneMap } from "../types";
import { updateMaterial } from "./updateMaterial";

export function addMaterial(material: MaterialJSON, map: SceneMap) {
  const materialObject = new MeshStandardMaterial();
  map.materials.set(material.id, materialObject);

  updateMaterial(material.id, material, map);
}
