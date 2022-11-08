import { MeshStandardMaterial } from "three";

import { MaterialJSON } from "../../../scene";
import { RenderWorker } from "../../RenderWorker";
import { SceneMap } from "../types";
import { updateMaterial } from "./updateMaterial";

export function addMaterial(
  material: MaterialJSON,
  map: SceneMap,
  renderWorker: RenderWorker
) {
  const materialObject = new MeshStandardMaterial();
  renderWorker.csm?.setupMaterial(materialObject);

  map.materials.set(material.id, materialObject);

  updateMaterial(material.id, material, map);
}
