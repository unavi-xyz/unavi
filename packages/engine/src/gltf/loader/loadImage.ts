import { fetchUri } from "../../utils/fetchUri";
import { GLTF } from "../schemaTypes";
import { BufferViewResult } from "./loadBufferView";

export async function loadImage(
  index: number,
  json: GLTF | null,
  baseUrl: string | null,
  loadBufferView: (index: number) => Promise<BufferViewResult>
): Promise<ImageBitmap> {
  if (!json) {
    throw new Error("No JSON found");
  }

  if (json.images === undefined) {
    throw new Error("No images found");
  }

  const { uri, bufferView, mimeType } = json.images[index];

  if (uri === undefined) {
    if (bufferView === undefined) {
      throw new Error("No uri or buffView found");
    }

    // Load image from buffer
    const { bufferView: arrayBuffer } = await loadBufferView(bufferView);
    const blob = new Blob([arrayBuffer], { type: mimeType });
    const image = await createImageBitmap(blob);
    return image;
  }

  // Fetch image
  const res = await fetchUri(uri, baseUrl);

  if (!res.ok) {
    throw new Error(`Failed to fetch ${uri}`);
  }

  const blob = await res.blob();
  const image = await createImageBitmap(blob);
  return image;
}
