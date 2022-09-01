import { InterleavedBuffer, InterleavedBufferAttribute } from "three";

import { ATTRIBUTE_TYPES, WEBGL_CONSTANTS } from "../constants";
import { BufferView, GLTF } from "../schemaTypes";
import { getPaddedBufferSize } from "./getPaddedBufferSize";

export function processInterleavedBufferView(
  attribute: InterleavedBufferAttribute,
  interleavedBuffer: InterleavedBuffer,
  componentType: number,
  start: number,
  count: number,
  json: GLTF,
  processBuffer: (buffer: ArrayBuffer, name: string) => number
) {
  if (!json.bufferViews) json.bufferViews = [];

  // Create a data view and add all attributes to it
  let componentSize: number;
  switch (componentType) {
    case WEBGL_CONSTANTS.UNSIGNED_BYTE:
      componentSize = 1;
      break;
    case WEBGL_CONSTANTS.UNSIGNED_SHORT:
      componentSize = 2;
      break;
    default:
      componentSize = 4;
  }

  const byteLength = getPaddedBufferSize(
    interleavedBuffer.count * interleavedBuffer.stride * componentSize
  );

  const dataView = new DataView(new ArrayBuffer(byteLength));

  let offset = 0;

  for (let i = 0; i < interleavedBuffer.array.length; i++) {
    const value = interleavedBuffer.array[i];

    switch (componentType) {
      case WEBGL_CONSTANTS.FLOAT:
        dataView.setFloat32(offset, value, true);
        break;
      case WEBGL_CONSTANTS.UNSIGNED_INT:
        dataView.setUint32(offset, value, true);
        break;
      case WEBGL_CONSTANTS.UNSIGNED_SHORT:
        dataView.setUint16(offset, value, true);
        break;
      case WEBGL_CONSTANTS.UNSIGNED_BYTE:
        dataView.setUint8(offset, value);
        break;
    }

    offset += componentSize;
  }

  // @ts-ignore
  const type: string = ATTRIBUTE_TYPES[attribute.itemSize];
  const componentName = getComponentName(componentType);
  let name = `${type}_${componentName}_${json.bufferViews.length}`;

  name = `${attribute.data.uuid}`;

  // See if we have already added this interleaved buffer
  const foundIndex = json.bufferViews.findIndex(
    (bufferView) => bufferView.name === name
  );
  if (foundIndex !== undefined && foundIndex !== -1) {
    return { index: foundIndex, bufferIndex: 0 };
  }

  const bufferIndex = processBuffer(dataView.buffer, name);

  const bufferViewDef: BufferView = {
    name,
    buffer: 0,
    byteLength,
  };

  const stride = attribute.data.stride;
  bufferViewDef.byteStride = stride * componentSize;

  const index = json.bufferViews.push(bufferViewDef) - 1;
  return { index, bufferIndex };
}

function getComponentName(componentType: number) {
  // @ts-ignore
  return Object.keys(WEBGL_CONSTANTS).find(
    (key) => WEBGL_CONSTANTS[key] === componentType
  );
}
