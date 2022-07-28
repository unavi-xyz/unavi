import { BufferAttribute, BufferGeometry, InterleavedBufferAttribute } from "three";

import { WEBGL_CONSTANTS } from "../constants";
import { Accessor, GLTF } from "../schemaTypes";
import { getMinMax } from "./getMinMax";

const TYPES = {
  1: "SCALAR",
  2: "VEC2",
  3: "VEC3",
  4: "VEC4",
  16: "MAT4",
};

export function processAccessor(
  attribute: BufferAttribute | InterleavedBufferAttribute,
  json: GLTF,
  processBufferView: (
    attribute: BufferAttribute | InterleavedBufferAttribute,
    componentType: number,
    start: number,
    count: number
  ) => number,
  geometry?: BufferGeometry,
  count?: number,
  start = 0
) {
  let componentType: number;

  switch (attribute.array.constructor) {
    case Float32Array:
      componentType = WEBGL_CONSTANTS.FLOAT;
      break;
    case Uint32Array:
      componentType = WEBGL_CONSTANTS.UNSIGNED_INT;
      break;
    case Uint16Array:
      componentType = WEBGL_CONSTANTS.UNSIGNED_SHORT;
      break;
    case Uint8Array:
      componentType = WEBGL_CONSTANTS.UNSIGNED_BYTE;
      break;
    default:
      throw new Error(`Unsupported component type: ${attribute.array.constructor.name}`);
  }

  if (count === undefined) count = attribute.count;
  if (count === 0) return null;

  // Truncate draw range using geometry if available
  if (geometry && geometry.index === null) {
    const end = start + count;
    const end2 =
      geometry.drawRange.count === Infinity
        ? attribute.count
        : geometry.drawRange.count + geometry.drawRange.start;

    start = Math.max(start, geometry.drawRange.start);
    count = Math.min(end, end2) - start;

    if (count < 0) count = 0;
  }

  const minMax = getMinMax(attribute, start, count);
  let bufferViewTarget: number | undefined;

  if (geometry) {
    if (attribute === geometry.index) {
      bufferViewTarget = WEBGL_CONSTANTS.ELEMENT_ARRAY_BUFFER;
    } else {
      bufferViewTarget = WEBGL_CONSTANTS.ARRAY_BUFFER;
    }
  }

  const bufferView = processBufferView(attribute, componentType, start, count);

  // @ts-ignore
  const type: string | undefined = TYPES[attribute.itemSize];
  if (type === undefined) throw new Error(`Unsupported item size: ${attribute.itemSize}`);

  const accessorDef: Accessor = {
    bufferView,
    componentType,
    count,
    max: minMax.max,
    min: minMax.min,
    type,
  };

  if (attribute.normalized) accessorDef.normalized = true;

  if (!json.accessors) json.accessors = [];
  const accessorIndex = json.accessors.push(accessorDef) - 1;
  return accessorIndex;
}
