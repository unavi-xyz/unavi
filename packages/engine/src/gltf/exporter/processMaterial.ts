import {
  DoubleSide,
  LineBasicMaterial,
  LineDashedMaterial,
  Material,
  MeshBasicMaterial,
  MeshLambertMaterial,
  MeshMatcapMaterial,
  MeshPhongMaterial,
  MeshPhysicalMaterial,
  MeshStandardMaterial,
  MeshToonMaterial,
  PointsMaterial,
  ShadowMaterial,
  SpriteMaterial,
  Texture,
} from "three";

import { arraysEqual } from "../../utils/arraysEqual";
import { GLTF, Material as MaterialDef, TextureInfo } from "../schemaTypes";
import { buildMetalRoughTexture } from "./buildMetalRoughTexture";

export function processMaterial(
  material: Material,
  json: GLTF,
  processTexture: (map: Texture) => number
) {
  if (
    !(
      material instanceof MeshStandardMaterial ||
      material instanceof MeshBasicMaterial ||
      material instanceof LineBasicMaterial ||
      material instanceof MeshPhysicalMaterial ||
      material instanceof MeshPhongMaterial ||
      material instanceof MeshLambertMaterial ||
      material instanceof MeshMatcapMaterial ||
      material instanceof MeshToonMaterial ||
      material instanceof PointsMaterial ||
      material instanceof ShadowMaterial ||
      material instanceof SpriteMaterial ||
      material instanceof LineDashedMaterial
    )
  ) {
    throw new Error("Material not supported.");
  }

  const hasMetalRoughness = "metalness" in material || "roughness" in material;
  const materialDef: MaterialDef = {};
  if (material.name) materialDef.name = material.name;
  materialDef.pbrMetallicRoughness = {};

  // Base color
  const color = material.color.toArray().concat(material.opacity);
  if (!arraysEqual(color, [1, 1, 1, 1])) {
    materialDef.pbrMetallicRoughness.baseColorFactor = color;
  }

  // Base color texture
  if ("map" in material && material.map) {
    const index = processTexture(material.map);
    materialDef.pbrMetallicRoughness.baseColorTexture = { index };
  }

  // Metallic roughness
  if (hasMetalRoughness) {
    materialDef.pbrMetallicRoughness.metallicFactor = material.metalness;
    materialDef.pbrMetallicRoughness.roughnessFactor = material.roughness;
  } else {
    materialDef.pbrMetallicRoughness.metallicFactor = 0.5;
    materialDef.pbrMetallicRoughness.roughnessFactor = 0.5;
  }

  // Metallic roughness texture
  if (hasMetalRoughness && (material.metalnessMap || material.roughnessMap)) {
    const texture = buildMetalRoughTexture(
      material.metalnessMap,
      material.roughnessMap
    );
    if (!texture) throw new Error("Failed to build metal roughness texture.");
    const index = processTexture(texture);
    materialDef.pbrMetallicRoughness.metallicRoughnessTexture = { index };
  }

  // Emissive
  if ("emissive" in material && material.emissive) {
    // Keep emissive between 0 and 1
    const emissive = material.emissive
      .clone()
      .multiplyScalar(material.emissiveIntensity);
    const maxEmissive = Math.max(...emissive.toArray());

    if (maxEmissive > 1) emissive.multiplyScalar(1 / maxEmissive);
    if (maxEmissive > 0) materialDef.emissiveFactor = emissive.toArray();

    // Emissive texture
    if (material.emissiveMap) {
      const index = processTexture(material.emissiveMap);
      materialDef.emissiveTexture = { index };
    }
  }

  // Normal texture
  if ("normalMap" in material && material.normalMap) {
    const index = processTexture(material.normalMap);
    const normalMapDef: TextureInfo = { index };

    if (material.normalScale && material.normalScale.x !== 1) {
      // Ignore y, because it may be flipped
      normalMapDef.scale = material.normalScale.x;
    }

    materialDef.normalTexture = normalMapDef;
  }

  // Occlusion texture
  if ("aoMap" in material && material.aoMap) {
    const index = processTexture(material.aoMap);
    materialDef.occlusionTexture = {
      index,
      texCoord: 1,
      strength: material.aoMapIntensity,
    };
  }

  // Alpha mode
  if (material.transparent) {
    materialDef.alphaMode = "BLEND";
  } else {
    if (material.alphaTest > 0) {
      materialDef.alphaMode = "MASK";
      materialDef.alphaCutoff = material.alphaTest;
    }
  }

  if (material.side === DoubleSide) materialDef.doubleSided = true;

  if (!json.materials) json.materials = [];
  const index = json.materials.push(materialDef) - 1;
  return index;
}
