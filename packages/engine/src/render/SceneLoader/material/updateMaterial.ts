import { Color, DoubleSide, FrontSide, sRGBEncoding } from "three";

import { MaterialJSON } from "../../../scene";
import { SceneMap } from "../types";
import { createTexture } from "./createTexture";

export async function updateMaterial(
  materialId: string,
  material: Partial<MaterialJSON>,
  map: SceneMap
) {
  const materialObject = map.materials.get(materialId);
  if (!materialObject) throw new Error("Material not found");

  if (material.alpha !== undefined) materialObject.opacity = material.alpha;

  if (material.alphaMode !== undefined) {
    if (material.alphaMode === "BLEND") {
      materialObject.transparent = true;
      materialObject.depthWrite = false;
    } else {
      materialObject.transparent = false;
      materialObject.depthWrite = true;

      if (material.alphaMode === "MASK") {
        if (material.alphaCutoff !== undefined)
          materialObject.alphaTest = material.alphaCutoff;
      }
    }
  }

  if (material.doubleSided !== undefined)
    materialObject.side = material.doubleSided ? DoubleSide : FrontSide;

  if (material.emissive !== undefined)
    materialObject.emissive = new Color().fromArray(material.emissive);

  if (material.normalScale !== undefined)
    materialObject.normalScale.set(material.normalScale, material.normalScale);

  if (material.occlusionStrength !== undefined)
    materialObject.aoMapIntensity = material.occlusionStrength;

  if (material.color !== undefined)
    materialObject.color = new Color().fromArray(material.color);

  if (material.roughness !== undefined)
    materialObject.roughness = material.roughness;

  if (material.metalness !== undefined)
    materialObject.metalness = material.metalness;

  if (material.colorTexture !== undefined) {
    const colorTexture = material.colorTexture
      ? await createTexture(material.colorTexture, map)
      : null;

    if (colorTexture) colorTexture.encoding = sRGBEncoding;

    materialObject.map = colorTexture;
  }

  if (material.normalTexture !== undefined) {
    const normalTexture = material.normalTexture
      ? await createTexture(material.normalTexture, map)
      : null;

    materialObject.normalMap = normalTexture;
  }

  if (material.occlusionTexture !== undefined) {
    const occlusionTexture = material.occlusionTexture
      ? await createTexture(material.occlusionTexture, map)
      : null;

    materialObject.aoMap = occlusionTexture;

    if (material.occlusionStrength !== undefined)
      materialObject.aoMapIntensity = material.occlusionStrength;
  }

  if (material.emissiveTexture !== undefined) {
    const emissiveTexture = material.emissiveTexture
      ? await createTexture(material.emissiveTexture, map)
      : null;

    if (emissiveTexture) emissiveTexture.encoding = sRGBEncoding;

    materialObject.emissiveMap = emissiveTexture;
  }

  if (material.metallicRoughnessTexture !== undefined) {
    const metallicRougnessTexture = material.metallicRoughnessTexture
      ? await createTexture(material.metallicRoughnessTexture, map)
      : null;

    materialObject.metalnessMap = metallicRougnessTexture;
    materialObject.roughnessMap = metallicRougnessTexture;
  }

  materialObject.needsUpdate = true;
}
