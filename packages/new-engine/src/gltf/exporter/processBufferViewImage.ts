import { BufferView, GLTF } from "../schemaTypes";
import { getPaddedArrayBuffer } from "./getPaddedArrayBuffer";
import { BufferViewResult } from "./processBufferView";

export async function processBufferViewImage(
  blob: Blob,
  json: GLTF,
  processBuffer: (buffer: ArrayBuffer, name: string) => number
): Promise<BufferViewResult> {
  if (!json.bufferViews) json.bufferViews = [];

  const name = `image_${json.bufferViews.length}`;

  const arrayBuffer = await blob.arrayBuffer();
  const buffer = getPaddedArrayBuffer(arrayBuffer);
  const bufferIndex = processBuffer(buffer, name);

  const bufferViewDef: BufferView = {
    name,
    buffer: 0,
    byteLength: buffer.byteLength,
  };

  const index = json.bufferViews.push(bufferViewDef) - 1;
  return { index, bufferIndex };
}
