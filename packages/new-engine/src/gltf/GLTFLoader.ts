import { fetchUri } from "../utils/fetchUri";
import { UriType, parseUri } from "../utils/parseUri";
import { GLTF } from "./schemaTypes";
import { LoadedBufferView, LoadedGLTF } from "./types";

const GLTF_MAGIC = "glTF";
const BINARY_HEADER_LENGTH = 12;
const BINARY_CHUNK_TYPES = { JSON: 0x4e4f534a, BIN: 0x004e4942 };

const decoder = new TextDecoder();

// Fetches a glTF and loads it into memory
// Designed to run in a Web Worker
export class GLTFLoader {
  private _json: GLTF | null = null;
  private _bin: ArrayBuffer | null = null;
  private _baseUrl: string | null = null;
  private _buffers = new Map<number, ArrayBuffer>();
  private _bufferViews = new Map<number, LoadedBufferView>();

  public async load(uri: string): Promise<LoadedGLTF> {
    // Parse uri
    const uriType = parseUri(uri);

    if (uriType === UriType.RelativePath) {
      // Save the base url
      // Append it to all future relative uris
      const split = uri.split("/");
      split.pop();
      if (split.length > 0) {
        this._baseUrl = split.join("/");
      }
    }

    // Fetch glTF
    const res = await fetchUri(uri);

    if (!res.ok) {
      throw new Error(`Failed to fetch ${uri}`);
    }

    const blob = await res.blob();

    try {
      // See if the file is in json format
      const text = await blob.text();
      this._json = JSON.parse(text);
    } catch {
      // If not, assume it's a binary glb
      const buffer = await blob.arrayBuffer();

      // Load header
      const headerView = new DataView(buffer, 0, BINARY_HEADER_LENGTH);

      const magic = decoder.decode(new Uint8Array(buffer.slice(0, 4)));
      const version = headerView.getUint32(4, true);
      const length = headerView.getUint32(8, true);

      if (magic !== GLTF_MAGIC) {
        throw new Error("Unsupported glTF magic");
      }

      if (version !== 2) {
        throw new Error("Unsupported glTF version");
      }

      // Load body
      const chunkLength = length - BINARY_HEADER_LENGTH;
      const chunkView = new DataView(buffer, BINARY_HEADER_LENGTH);

      let chunkIndex = 0;

      while (chunkIndex < chunkLength) {
        const chunkLength = chunkView.getUint32(chunkIndex, true);
        chunkIndex += 4;

        const chunkType = chunkView.getUint32(chunkIndex, true);
        chunkIndex += 4;

        const byteOffset = BINARY_HEADER_LENGTH + chunkIndex;

        switch (chunkType) {
          case BINARY_CHUNK_TYPES.JSON:
            const jsonArray = new Uint8Array(buffer, byteOffset, chunkLength);
            const text = decoder.decode(jsonArray);
            this._json = JSON.parse(text);
            break;
          case BINARY_CHUNK_TYPES.BIN:
            this._bin = buffer.slice(byteOffset, byteOffset + chunkLength);
            break;
          default:
            break;
        }

        chunkIndex += chunkLength;
      }
    }

    if (!this._json) {
      throw new Error("No JSON found");
    }

    // Load buffers
    const bufferPromises =
      this._json.buffers?.map((_, index) => {
        return this._loadBuffer(index);
      }) ?? [];

    await Promise.all(bufferPromises);

    // Load buffer views
    const bufferViewPromises =
      this._json.bufferViews?.map((_, index) => {
        return this._loadBufferView(index);
      }) ?? [];

    const bufferViews = await Promise.all(bufferViewPromises);

    // Load images
    const imagePromises =
      this._json.images?.map(async (_, index) => {
        const image = await this._loadImage(index);
        return image;
      }) ?? [];

    const images = await Promise.all(imagePromises);

    const json = { ...this._json };

    // Clear cache
    this._buffers.clear();
    this._bufferViews.clear();
    this._json = null;
    this._bin = null;
    this._baseUrl = null;

    return {
      json,
      bufferViews,
      images,
    };
  }

  private async _loadImage(index: number) {
    if (this._json?.images === undefined) {
      throw new Error("No images found");
    }

    const { uri, bufferView, mimeType } = this._json.images[index];

    if (uri === undefined) {
      if (bufferView === undefined) {
        throw new Error("No uri or buffView found");
      }

      // Load image from buffer
      const { bufferView: arrayBuffer } = await this._loadBufferView(bufferView);
      const blob = new Blob([arrayBuffer], { type: mimeType });
      const image = await createImageBitmap(blob);
      return image;
    }

    // Fetch image
    const res = await fetchUri(uri, this._baseUrl);

    if (!res.ok) {
      throw new Error(`Failed to fetch ${uri}`);
    }

    const blob = await res.blob();
    const image = await createImageBitmap(blob);
    return image;
  }

  private async _loadBufferView(index: number) {
    const cached = this._bufferViews.get(index);
    if (cached) return cached;

    if (this._json?.bufferViews === undefined) {
      throw new Error("No bufferViews found");
    }

    const bufferViewDef = this._json.bufferViews[index];
    const buffer = await this._loadBuffer(bufferViewDef.buffer);

    const byteLength = bufferViewDef.byteLength;
    const byteOffset = bufferViewDef.byteOffset ?? 0;
    const bufferView = buffer.slice(byteOffset, byteOffset + byteLength);

    const result = { bufferViewDef, bufferView };
    this._bufferViews.set(index, result);
    return result;
  }

  private async _loadBuffer(index: number) {
    const cached = this._buffers.get(index);
    if (cached) return cached;

    if (this._json?.buffers === undefined) {
      throw new Error("No buffers found");
    }

    const { uri } = this._json.buffers[index];

    // If present, GLB bin is required to be the first buffer
    if (uri === undefined && index === 0) {
      if (!this._bin) {
        throw new Error("No bin found");
      }

      return this._bin;
    }

    if (uri === undefined) {
      throw new Error("No uri found");
    }

    // Fetch buffer
    const res = await fetchUri(uri, this._baseUrl);

    if (!res.ok) {
      throw new Error(`Failed to fetch ${uri}`);
    }

    const arrayBuffer = await res.arrayBuffer();
    this._buffers.set(index, arrayBuffer);
    return arrayBuffer;
  }
}
