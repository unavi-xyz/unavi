import { BufferView, GLTF } from "../schemaTypes";
import { getPaddedArrayBuffer } from "./getPaddedArrayBuffer";

interface BufferViewImageResult {
  index: number;
  byteLength: number;
}

export async function processBufferViewImage(
  blob: Blob,
  json: GLTF,
  byteOffset: number,
  processBuffer: (buffer: ArrayBuffer) => number
): Promise<BufferViewImageResult> {
  const arrayBuffer = await blob.arrayBuffer();
  const buffer = getPaddedArrayBuffer(arrayBuffer);
  const bufferIndex = processBuffer(buffer);

  const bufferViewDef: BufferView = {
    buffer: bufferIndex,
    byteOffset,
    byteLength: buffer.byteLength,
  };

  if (!json.bufferViews) json.bufferViews = [];
  const index = json.bufferViews.push(bufferViewDef) - 1;
  return { index, byteLength: buffer.byteLength };
}
