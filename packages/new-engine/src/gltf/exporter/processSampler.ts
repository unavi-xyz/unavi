import {
  ClampToEdgeWrapping,
  LinearFilter,
  LinearMipmapLinearFilter,
  LinearMipmapNearestFilter,
  MirroredRepeatWrapping,
  NearestFilter,
  NearestMipmapLinearFilter,
  NearestMipmapNearestFilter,
  RepeatWrapping,
  Texture,
  TextureFilter,
  Wrapping,
} from "three";

import { WEBGL_CONSTANTS } from "../constants";
import { GLTF, Sampler } from "../schemaTypes";

const FILTER = new Map<TextureFilter, number>();
FILTER.set(NearestFilter, WEBGL_CONSTANTS.NEAREST);
FILTER.set(NearestMipmapNearestFilter, WEBGL_CONSTANTS.NEAREST_MIPMAP_NEAREST);
FILTER.set(NearestMipmapLinearFilter, WEBGL_CONSTANTS.NEAREST_MIPMAP_LINEAR);
FILTER.set(LinearFilter, WEBGL_CONSTANTS.LINEAR);
FILTER.set(LinearMipmapNearestFilter, WEBGL_CONSTANTS.LINEAR_MIPMAP_NEAREST);
FILTER.set(LinearMipmapLinearFilter, WEBGL_CONSTANTS.LINEAR_MIPMAP_LINEAR);

const WRAP = new Map<Wrapping, number>();
WRAP.set(ClampToEdgeWrapping, WEBGL_CONSTANTS.CLAMP_TO_EDGE);
WRAP.set(RepeatWrapping, WEBGL_CONSTANTS.REPEAT);
WRAP.set(MirroredRepeatWrapping, WEBGL_CONSTANTS.MIRRORED_REPEAT);

export function processSampler(map: Texture, json: GLTF) {
  const samplerDef: Sampler = {
    magFilter: FILTER.get(map.magFilter),
    minFilter: FILTER.get(map.minFilter),
    wrapS: WRAP.get(map.wrapS),
    wrapT: WRAP.get(map.wrapT),
  };

  if (!json.samplers) json.samplers = [];

  // Check if sampler already exists
  const stringified = JSON.stringify(samplerDef);
  const existing = json.samplers.findIndex((sampler) => JSON.stringify(sampler) === stringified);
  if (existing !== -1) return existing;

  // Otherwise add a new sampler
  const index = json.samplers.push(samplerDef) - 1;
  return index;
}
