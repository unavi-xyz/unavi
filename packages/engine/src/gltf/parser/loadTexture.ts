import { CanvasTexture, LinearFilter, LinearMipMapLinearFilter, RepeatWrapping } from "three";

import { WEBGL_FILTERS, WEBGL_WRAPPINGS } from "../constants";
import {
  GLTF,
  MaterialNormalTextureInfo,
  MaterialOcclusionTextureInfo,
  Sampler,
  TextureInfo,
} from "../schemaTypes";

export async function loadTexture(
  info: TextureInfo | MaterialNormalTextureInfo | MaterialOcclusionTextureInfo,
  json: GLTF,
  images: ImageBitmap[]
) {
  if (json.textures === undefined) {
    throw new Error("No textures found");
  }

  const textureDef = json.textures[info.index];

  if (textureDef.source === undefined) {
    throw new Error(`Texture ${info.index} has no source`);
  }

  // Create texture
  const image = images[textureDef.source];
  const texture = new CanvasTexture(image);
  texture.needsUpdate = true;
  texture.flipY = false;
  if (textureDef.name) texture.name = textureDef.name;

  // Sampler
  let samplerDef: Sampler = {};
  const samplerIndex = textureDef.sampler;
  if (samplerIndex !== undefined) {
    if (json.samplers === undefined) {
      throw new Error("No samplers found");
    }
    samplerDef = json.samplers[samplerIndex];
  }

  const magFilter = samplerDef.magFilter as keyof typeof WEBGL_FILTERS;
  texture.magFilter = WEBGL_FILTERS[magFilter] ?? LinearFilter;

  const minFilter = samplerDef.minFilter as keyof typeof WEBGL_FILTERS;
  texture.minFilter = WEBGL_FILTERS[minFilter] ?? LinearMipMapLinearFilter;

  const wrapS = samplerDef.wrapS as keyof typeof WEBGL_WRAPPINGS;
  texture.wrapS = WEBGL_WRAPPINGS[wrapS] ?? RepeatWrapping;

  const wrapT = samplerDef.wrapT as keyof typeof WEBGL_WRAPPINGS;
  texture.wrapT = WEBGL_WRAPPINGS[wrapT] ?? RepeatWrapping;

  return texture;
}
