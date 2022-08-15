import { BufferView, GLTF } from "../schemaTypes";

export type BufferViewResult = {
  bufferView: ArrayBuffer;
  bufferViewDef: BufferView;
};

export async function loadBufferView(
  index: number,
  json: GLTF | null,
  loadBuffer: (index: number) => Promise<ArrayBuffer>
): Promise<BufferViewResult> {
  if (!json) {
    throw new Error("No JSON found");
  }

  if (json.bufferViews === undefined) {
    throw new Error("No bufferViews found");
  }

  const bufferViewDef = json.bufferViews[index];
  const buffer = await loadBuffer(bufferViewDef.buffer);

  const byteLength = bufferViewDef.byteLength;
  const byteOffset = bufferViewDef.byteOffset ?? 0;
  const bufferView = buffer.slice(byteOffset, byteOffset + byteLength);

  return { bufferViewDef, bufferView };
}
