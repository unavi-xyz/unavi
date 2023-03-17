import { Texture } from "@gltf-transform/core";
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

import { TextureInfoJSON } from "../../../scene/attributes/TextureInfoUtils";
import { WEBGL_CONSTANTS } from "../../constants";

export async function createTexture(
  texture: Texture | null,
  info: TextureInfoJSON | null
): Promise<ThreeTexture | null> {
  if (!texture) return null;

  const array = texture.getImage();
  if (!array) throw new Error("Texture image is null");

  const mimeType = texture.getMimeType();
  const blob = new Blob([array], { type: mimeType });
  const bitmap = await createImageBitmap(blob);

  const object = new CanvasTexture(bitmap);
  object.needsUpdate = true;

  if (info?.offset) object.offset.fromArray(info.offset);
  if (info?.rotation) object.rotation = info.rotation;
  if (info?.scale) object.repeat.fromArray(info.scale);

  switch (info?.magFilter) {
    case WEBGL_CONSTANTS.NEAREST: {
      object.magFilter = NearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR:
    default: {
      object.magFilter = LinearFilter;
      break;
    }
  }

  switch (info?.minFilter) {
    case WEBGL_CONSTANTS.NEAREST: {
      object.minFilter = NearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.NEAREST_MIPMAP_NEAREST: {
      object.minFilter = NearestMipMapNearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.NEAREST_MIPMAP_LINEAR: {
      object.minFilter = NearestMipMapLinearFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR_MIPMAP_NEAREST: {
      object.minFilter = LinearMipMapNearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR_MIPMAP_LINEAR: {
      object.minFilter = LinearMipMapLinearFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR:
    default: {
      object.minFilter = LinearFilter;
      break;
    }
  }

  switch (info?.wrapS) {
    case WEBGL_CONSTANTS.CLAMP_TO_EDGE: {
      object.wrapS = ClampToEdgeWrapping;
      break;
    }

    case WEBGL_CONSTANTS.MIRRORED_REPEAT: {
      object.wrapS = MirroredRepeatWrapping;
      break;
    }

    case WEBGL_CONSTANTS.REPEAT:
    default: {
      object.wrapS = RepeatWrapping;
      break;
    }
  }

  switch (info?.wrapT) {
    case WEBGL_CONSTANTS.CLAMP_TO_EDGE: {
      object.wrapT = ClampToEdgeWrapping;
      break;
    }

    case WEBGL_CONSTANTS.MIRRORED_REPEAT: {
      object.wrapT = MirroredRepeatWrapping;
      break;
    }

    case WEBGL_CONSTANTS.REPEAT:
    default: {
      object.wrapT = RepeatWrapping;
      break;
    }
  }

  return object;
}
