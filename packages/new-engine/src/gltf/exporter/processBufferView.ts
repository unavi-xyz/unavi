import { BufferAttribute, InterleavedBufferAttribute } from "three";

import { ATTRIBUTE_TYPES, WEBGL_CONSTANTS } from "../constants";
import { BufferView, GLTF } from "../schemaTypes";
import { getPaddedBufferSize } from "./getPaddedBufferSize";

export type BufferViewResult = {
  index: number;
  bufferIndex: number;
};

export function processBufferView(
  attribute: BufferAttribute | InterleavedBufferAttribute,
  componentType: number,
  start: number,
  count: number,
  json: GLTF,
  processBuffer: (buffer: ArrayBuffer, name: string) => number
) {
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

  const byteLength = getPaddedBufferSize(count * attribute.itemSize * componentSize);
  const dataView = new DataView(new ArrayBuffer(byteLength));

  let offset = 0;

  for (let i = 0; i < start + count; i++) {
    for (let j = 0; j < attribute.itemSize; j++) {
      let value: number | undefined = undefined;

      if (attribute.itemSize > 4) {
        // no support for interleaved data for itemSize > 4
        value = attribute.array[i * attribute.itemSize + j];
      } else {
        switch (j) {
          case 0:
            value = attribute.getX(i);
            break;
          case 1:
            value = attribute.getY(i);
            break;
          case 2:
            value = attribute.getZ(i);
            break;
          case 3:
            value = attribute.getW(i);
            break;
        }
      }

      if (value === undefined) throw new Error(`Value is undefined`);

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
  }
  // @ts-ignore
  const type: string = ATTRIBUTE_TYPES[attribute.itemSize];
  const componentName = getComponentName(componentType);
  const name = `${type}_${componentName}`;

  const bufferIndex = processBuffer(dataView.buffer, name);

  // See if we can combine this bufferView into a previous bufferView
  // We look for a bufferView with the same attribute and component type
  // We store this info in the name
  const foundIndex = json.bufferViews?.findIndex((bufferView) => bufferView.name === name);

  if (!json.bufferViews) json.bufferViews = [];
  if (!json.buffers) throw new Error(`No buffers found`);

  if (foundIndex !== undefined && foundIndex !== -1) {
    const found = json.bufferViews[foundIndex];
    found.byteLength += byteLength;
    return { index: foundIndex, bufferIndex };
  }

  const bufferViewDef: BufferView = {
    name,
    buffer: 0,
    byteLength,
  };

  const index = json.bufferViews.push(bufferViewDef) - 1;
  return { index, bufferIndex };
}

function getComponentName(componentType: number) {
  // @ts-ignore
  return Object.keys(WEBGL_CONSTANTS).find((key) => WEBGL_CONSTANTS[key] === componentType);
}
