import { fetchUri } from "../utils/fetchUri";
import { UriType, parseUri } from "../utils/parseUri";
import { loadBuffer } from "./loader/loadBuffer";
import { BufferViewResult, loadBufferView } from "./loader/loadBufferView";
import { loadImage } from "./loader/loadImage";
import { GLTF } from "./schemaTypes";
import { LoadedGLTF } from "./types";

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

  // Cache
  private _buffers = new Map<number, Promise<ArrayBuffer>>();
  private _bufferViews = new Map<number, Promise<BufferViewResult>>();
  private _images = new Map<number, Promise<ImageBitmap>>();

  public async load(uri: string): Promise<LoadedGLTF> {
    // Parse uri
    const uriType = parseUri(uri);

    if (uriType === UriType.RelativePath) {
      // Save the base url
      // Append it to all future relative urls
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

    // Load buffer views
    const bufferViewPromises =
      this._json.bufferViews?.map((_, index) => this._loadBufferView(index)) ?? [];

    // Load images
    const imagePromises = this._json.images?.map((_, index) => this._loadImage(index)) ?? [];

    const bufferViews = await Promise.all(bufferViewPromises);
    const images = await Promise.all(imagePromises);

    // Clear cache
    this._buffers.clear();
    this._bufferViews.clear();
    this._images.clear();

    return {
      json: this._json,
      bufferViews,
      images,
    };
  }

  private _loadImage(index: number) {
    const cached = this._images.get(index);
    if (cached) return cached;

    const image = loadImage(index, this._json, this._baseUrl, this._loadBufferView.bind(this));

    this._images.set(index, image);
    return image;
  }

  private _loadBufferView(index: number) {
    const cached = this._bufferViews.get(index);
    if (cached) return cached;

    const bufferView = loadBufferView(index, this._json, this._loadBuffer.bind(this));

    this._bufferViews.set(index, bufferView);
    return bufferView;
  }

  private _loadBuffer(index: number) {
    const cached = this._buffers.get(index);
    if (cached) return cached;

    const buffer = loadBuffer(index, this._json, this._bin, this._baseUrl);

    this._buffers.set(index, buffer);
    return buffer;
  }
}
