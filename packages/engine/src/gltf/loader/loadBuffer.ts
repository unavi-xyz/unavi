import { fetchUri } from "../../utils/fetchUri";
import { GLTF } from "../schemaTypes";

export async function loadBuffer(
  index: number,
  json: GLTF | null,
  bin: ArrayBuffer | null,
  baseUrl: string | null
): Promise<ArrayBuffer> {
  if (!json) {
    throw new Error("No JSON found");
  }

  if (json.buffers === undefined) {
    throw new Error("No buffers found");
  }

  const { uri } = json.buffers[index];

  // If present, GLB bin is required to be the first buffer
  if (uri === undefined && index === 0) {
    if (!bin) {
      throw new Error("No bin found");
    }

    return bin;
  }

  if (uri === undefined) {
    throw new Error("No uri found");
  }

  // Fetch buffer
  const res = await fetchUri(uri, baseUrl);

  if (!res.ok) {
    throw new Error(`Failed to fetch ${uri}`);
  }

  const arrayBuffer = await res.arrayBuffer();
  return arrayBuffer;
}
