import { MeshStandardMaterial } from "three";

import { MaterialJSON } from "../../../scene";
import { SceneMap } from "../types";
import { updateMaterial } from "./updateMaterial";

export async function addMaterial(material: MaterialJSON, map: SceneMap) {
  const materialObject = new MeshStandardMaterial();
  map.materials.set(material.id, materialObject);

  await updateMaterial(material.id, material, map);
}
