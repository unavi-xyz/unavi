import { BufferView, GLTF } from "../schemaTypes";
import { getPaddedArrayBuffer } from "./getPaddedArrayBuffer";

export async function processBufferViewImage(
  blob: Blob,
  json: GLTF,
  processBuffer: (buffer: ArrayBuffer, name: string) => number
): Promise<number> {
  if (!json.bufferViews) json.bufferViews = [];

  const name = `image_${json.bufferViews.length}`;

  const arrayBuffer = await blob.arrayBuffer();
  const buffer = getPaddedArrayBuffer(arrayBuffer);
  processBuffer(buffer, name);

  const bufferViewDef: BufferView = {
    name,
    buffer: 0,
    byteLength: buffer.byteLength,
  };

  const index = json.bufferViews.push(bufferViewDef) - 1;
  return index;
}
