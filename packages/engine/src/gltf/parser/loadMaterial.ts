import { DoubleSide, MeshStandardMaterial, sRGBEncoding, Texture } from "three";

import { ALPHA_MODES } from "../constants";
import {
  GLTF,
  MaterialNormalTextureInfo,
  MaterialOcclusionTextureInfo,
  TextureInfo,
} from "../schemaTypes";

export async function loadMaterial(
  index: number,
  json: GLTF,
  loadTexture: (
    info: TextureInfo | MaterialNormalTextureInfo | MaterialOcclusionTextureInfo
  ) => Promise<Texture>
) {
  if (json.materials === undefined) {
    throw new Error("No materials found");
  }

  const materialDef = json.materials[index];
  const material = new MeshStandardMaterial();
  material.name = materialDef.name ?? `material_${index}`;

  const {
    baseColorFactor = [1, 1, 1, 1],
    metallicFactor = 1,
    roughnessFactor = 1,
    baseColorTexture,
    metallicRoughnessTexture,
  } = materialDef.pbrMetallicRoughness ?? {};

  material.color.fromArray(baseColorFactor);
  material.opacity = baseColorFactor[3];
  material.metalness = metallicFactor;
  material.roughness = roughnessFactor;

  if (materialDef.doubleSided) material.side = DoubleSide;

  const emissiveFactor = materialDef.emissiveFactor ?? [0, 0, 0];
  material.emissive.fromArray(emissiveFactor);

  // Alpha
  const alphaMode = materialDef.alphaMode ?? ALPHA_MODES.OPAQUE;

  if (alphaMode === ALPHA_MODES.BLEND) {
    material.transparent = true;
    material.depthWrite = false;
  } else {
    material.transparent = false;

    if (alphaMode === ALPHA_MODES.MASK) {
      material.alphaTest = materialDef.alphaCutoff ?? 0.5;
    }
  }

  // Textures
  if (baseColorTexture) {
    const texture = await loadTexture(baseColorTexture);
    texture.encoding = sRGBEncoding;
    material.map = texture;
  }

  if (metallicRoughnessTexture) {
    const texture = await loadTexture(metallicRoughnessTexture);
    material.metalnessMap = texture;
    material.roughnessMap = texture;
  }

  if (materialDef.normalTexture) {
    const texture = await loadTexture(materialDef.normalTexture);
    material.normalMap = texture;
    const scale = materialDef.normalTexture.scale ?? 1;
    material.normalScale.set(scale, scale);
  }

  if (materialDef.occlusionTexture) {
    const texture = await loadTexture(materialDef.occlusionTexture);
    material.aoMap = texture;
    material.aoMapIntensity = materialDef.occlusionTexture.strength ?? 1;
  }

  if (materialDef.emissiveTexture) {
    const texture = await loadTexture(materialDef.emissiveTexture);
    texture.encoding = sRGBEncoding;
    material.emissiveMap = texture;
  }

  return material;
}
