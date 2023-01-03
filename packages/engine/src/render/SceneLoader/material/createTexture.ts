import {
  CanvasTexture,
  ClampToEdgeWrapping,
  LinearFilter,
  LinearMipMapLinearFilter,
  LinearMipMapNearestFilter,
  MirroredRepeatWrapping,
  NearestFilter,
  NearestMipMapLinearFilter,
  NearestMipMapNearestFilter,
  RepeatWrapping,
  Texture as ThreeTexture,
} from "three";

import { TextureJSON } from "../../../scene";
import { WEBGL_CONSTANTS } from "../../constants";
import { SceneMap } from "../types";

export function createTexture(
  { imageId, magFilter, minFilter, wrapS, wrapT, offset, rotation, scale }: TextureJSON,
  map: SceneMap
): ThreeTexture {
  if (imageId === null) throw new Error("Texture source not found");

  const image = map.images.get(imageId);
  if (!image) throw new Error("Image not found");

  const threeTexture = new CanvasTexture(image);
  threeTexture.needsUpdate = true;

  switch (magFilter) {
    case WEBGL_CONSTANTS.NEAREST:
      threeTexture.magFilter = NearestFilter;
      break;
    case WEBGL_CONSTANTS.LINEAR:
      threeTexture.magFilter = LinearFilter;
      break;
    default:
      throw new Error(`Unknown magFilter: ${magFilter}`);
  }

  switch (minFilter) {
    case WEBGL_CONSTANTS.NEAREST:
      threeTexture.minFilter = NearestFilter;
      break;
    case WEBGL_CONSTANTS.LINEAR:
      threeTexture.minFilter = LinearFilter;
      break;
    case WEBGL_CONSTANTS.NEAREST_MIPMAP_NEAREST:
      threeTexture.minFilter = NearestMipMapNearestFilter;
      break;
    case WEBGL_CONSTANTS.LINEAR_MIPMAP_NEAREST:
      threeTexture.minFilter = LinearMipMapNearestFilter;
      break;
    case WEBGL_CONSTANTS.NEAREST_MIPMAP_LINEAR:
      threeTexture.minFilter = NearestMipMapLinearFilter;
      break;
    case WEBGL_CONSTANTS.LINEAR_MIPMAP_LINEAR:
      threeTexture.minFilter = LinearMipMapLinearFilter;
      break;
    default:
      throw new Error(`Unknown minFilter: ${minFilter}`);
  }

  switch (wrapS) {
    case WEBGL_CONSTANTS.CLAMP_TO_EDGE:
      threeTexture.wrapS = ClampToEdgeWrapping;
      break;
    case WEBGL_CONSTANTS.MIRRORED_REPEAT:
      threeTexture.wrapS = MirroredRepeatWrapping;
      break;
    case WEBGL_CONSTANTS.REPEAT:
      threeTexture.wrapS = RepeatWrapping;
      break;
    default:
      throw new Error(`Unknown wrapS: ${wrapS}`);
  }

  switch (wrapT) {
    case WEBGL_CONSTANTS.CLAMP_TO_EDGE:
      threeTexture.wrapT = ClampToEdgeWrapping;
      break;
    case WEBGL_CONSTANTS.MIRRORED_REPEAT:
      threeTexture.wrapT = MirroredRepeatWrapping;
      break;
    case WEBGL_CONSTANTS.REPEAT:
      threeTexture.wrapT = RepeatWrapping;
      break;
    default:
      throw new Error(`Unknown wrapT: ${wrapT}`);
  }

  if (offset) threeTexture.offset.fromArray(offset);
  if (rotation) threeTexture.rotation = rotation;
  if (scale) threeTexture.repeat.fromArray(scale);

  return threeTexture;
}
