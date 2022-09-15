import { fetchUri } from "../utils/fetchUri";
import { parseUri, UriType } from "../utils/parseUri";
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
  #json: GLTF | null = null;
  #bin: ArrayBuffer | null = null;
  #baseUrl: string | null = null;

  // Cache
  #buffers = new Map<number, Promise<ArrayBuffer>>();
  #bufferViews = new Map<number, Promise<BufferViewResult>>();

  async load(uri: string): Promise<LoadedGLTF> {
    // Parse uri
    const uriType = parseUri(uri);

    if (uriType === UriType.RelativePath) {
      // Save the base url
      // Append it to all future relative urls
      const split = uri.split("/");
      split.pop();
      if (split.length > 0) {
        this.#baseUrl = split.join("/");
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
      this.#json = JSON.parse(text);
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
            this.#json = JSON.parse(text);
            break;
          case BINARY_CHUNK_TYPES.BIN:
            this.#bin = buffer.slice(byteOffset, byteOffset + chunkLength);
            break;
          default:
            break;
        }

        chunkIndex += chunkLength;
      }
    }

    if (!this.#json) {
      throw new Error("No JSON found");
    }

    console.info("🔃 Loaded glTF:", this.#json);

    const bufferViewPromises =
      this.#json.bufferViews?.map((_, index) => this.#loadBufferView(index)) ??
      [];

    const imagePromises =
      this.#json.images?.map((_, index) => this.#loadImage(index)) ?? [];

    const bufferViews = await Promise.all(bufferViewPromises);
    const images = await Promise.all(imagePromises);

    // Clear cache
    this.#buffers.clear();
    this.#bufferViews.clear();

    return {
      json: this.#json,
      bufferViews,
      images,
    };
  }

  #loadImage(index: number) {
    const image = loadImage(
      index,
      this.#json,
      this.#baseUrl,
      this.#loadBufferView.bind(this)
    );
    return image;
  }

  #loadBufferView(index: number) {
    const cached = this.#bufferViews.get(index);
    if (cached) return cached;

    const bufferView = loadBufferView(
      index,
      this.#json,
      this.#loadBuffer.bind(this)
    );

    this.#bufferViews.set(index, bufferView);
    return bufferView;
  }

  #loadBuffer(index: number) {
    const cached = this.#buffers.get(index);
    if (cached) return cached;

    const buffer = loadBuffer(index, this.#json, this.#bin, this.#baseUrl);

    this.#buffers.set(index, buffer);
    return buffer;
  }
}
