import {
  AnimationClip,
  BufferAttribute,
  BufferGeometry,
  InterleavedBufferAttribute,
  Line,
  Material,
  Mesh,
  Object3D,
  Points,
  Scene,
  SkinnedMesh,
  Texture,
} from "three";

import { getPaddedArrayBuffer } from "./exporter/getPaddedArrayBuffer";
import { processAccessor } from "./exporter/processAccessor";
import { processAnimation } from "./exporter/processAnimation";
import { processBufferView } from "./exporter/processBufferView";
import { processBufferViewImage } from "./exporter/processBufferViewImage";
import { processImage } from "./exporter/processImage";
import { processMaterial } from "./exporter/processMaterial";
import { processMesh } from "./exporter/processMesh";
import { processNode } from "./exporter/processNode";
import { processSampler } from "./exporter/processSampler";
import { processScene } from "./exporter/processScene";
import { processSkin } from "./exporter/processSkin";
import { processTexture } from "./exporter/processTexture";
import { GLTF } from "./schemaTypes";

const GLB_HEADER_BYTES = 12;
const GLB_HEADER_MAGIC = 0x46546c67;
const GLB_VERSION = 2;

const GLB_CHUNK_PREFIX_BYTES = 8;
const GLB_CHUNK_TYPE_JSON = 0x4e4f534a;
const GLB_CHUNK_TYPE_BIN = 0x004e4942;

export interface GLTFExporterOptions {
  binary: boolean;
}

// Converts a three.js scene to glTF
export class GLTFExporter {
  #options: GLTFExporterOptions = {
    binary: false,
  };

  #json: GLTF = {
    asset: {
      version: "2.0",
      generator: "Wired Engine",
    },
  };

  #buffers: ArrayBuffer[] = [];
  #byteOffset = 0;

  #skins: SkinnedMesh[] = [];

  #pending: Promise<any>[] = [];
  #cache = {
    images: new Map<ImageBitmap, Map<boolean, Map<string, number>>>(),
    materials: new Map<Material, number>(),
    meshes: new Map<Mesh | Line | Points, number | null>(),
    nodes: new Map<Object3D, number>(),
    textures: new Map<Texture, number>(),
  };

  constructor() {}

  async exportAsJSON(input: Object3D | Object3D[], animations?: AnimationClip[]) {
    this.#options.binary = true;
    const json = await this.#exportObjects(input, animations);
    return json as GLTF;
  }

  async exportAsBinary(input: Object3D | Object3D[], animations?: AnimationClip[]) {
    this.#options.binary = true;
    const glbBuffer = await this.#exportObjects(input, animations);
    return glbBuffer as ArrayBuffer;
  }

  async #exportObjects(input: Object3D | Object3D[], animations?: AnimationClip[]) {
    // Process input
    this.#processInput(input, animations);

    await Promise.all(this.#pending);

    // Merge buffers
    const blob = new Blob(this.#buffers, { type: "application/octet-stream" });

    // Update byte length of the single buffer
    if (this.#json.buffers && this.#json.buffers.length > 1) {
      this.#json.buffers[0].byteLength = blob.size;
    }

    if (this.#options.binary) {
      const buffer = await blob.arrayBuffer();

      // Binary chunk
      const binaryChunk = getPaddedArrayBuffer(buffer);
      const binaryChunkPrefix = new DataView(new ArrayBuffer(GLB_CHUNK_PREFIX_BYTES));
      binaryChunkPrefix.setUint32(0, binaryChunk.byteLength, true);
      binaryChunkPrefix.setUint32(4, GLB_CHUNK_TYPE_BIN, true);

      // JSON chunk
      const jsonText = JSON.stringify(this.#json);
      const jsonBuffer = new TextEncoder().encode(jsonText).buffer;
      const jsonChunk = getPaddedArrayBuffer(jsonBuffer, 0x20);
      const jsonChunkPrefix = new DataView(new ArrayBuffer(GLB_CHUNK_PREFIX_BYTES));
      jsonChunkPrefix.setUint32(0, jsonChunk.byteLength, true);
      jsonChunkPrefix.setUint32(4, GLB_CHUNK_TYPE_JSON, true);

      // GLB header
      const header = new ArrayBuffer(GLB_HEADER_BYTES);
      const headerView = new DataView(header);
      headerView.setUint32(0, GLB_HEADER_MAGIC, true);
      headerView.setUint32(4, GLB_VERSION, true);
      const totalByteLength =
        GLB_HEADER_BYTES +
        jsonChunkPrefix.byteLength +
        jsonChunk.byteLength +
        binaryChunkPrefix.byteLength +
        binaryChunk.byteLength;
      headerView.setUint32(8, totalByteLength, true);

      const glbBlob = new Blob(
        [header, jsonChunkPrefix, jsonChunk, binaryChunkPrefix, binaryChunk],
        { type: "application/octet-stream" }
      );

      const glbBuffer = await glbBlob.arrayBuffer();
      return glbBuffer;
    } else {
      if (this.#json.buffers && this.#json.buffers.length > 0) {
        // Embed buffer in json
        const text = await blob.text();
        this.#json.buffers[0].uri = `data:application/octet-stream;base64,${text}`;
      }

      return this.#json;
    }
  }

  #processInput(input: Object3D | Object3D[], animations: AnimationClip[] = []) {
    const objects = Array.isArray(input) ? input : [input];
    const objectsWithoutScene: Object3D[] = [];

    objects.forEach((object) => {
      if (object instanceof Scene) {
        // Process scenes
        this.#processScene(object);
      } else {
        objectsWithoutScene.push(object);
      }
    });

    // Process non-scene objects
    if (objectsWithoutScene.length > 0) {
      const scene = new Scene();
      // Push to children instead of adding to scene to avoid modifying .parent attribute
      scene.children.push(...objectsWithoutScene);
      this.#processScene(scene);
    }

    // Set default scene
    if (this.#json.scenes?.length === 1) this.#json.scene = 0;

    // Process animations
    animations.forEach((clip) => {
      this.#processAnimation(clip, objects[0]);
    });

    // Process skins
    this.#skins.forEach((skin) => {
      this.#processSkin(skin);
    });
  }

  #processAnimation(clip: AnimationClip, root: Object3D) {
    const index = processAnimation(
      clip,
      root,
      this.#json,
      this.#processNode.bind(this),
      this.#processAccessor.bind(this)
    );
    return index;
  }

  #processSkin(skin: SkinnedMesh) {
    const index = processSkin(
      skin,
      this.#json,
      this.#processNode.bind(this),
      this.#processAccessor.bind(this)
    );
    return index;
  }

  #processScene(scene: Scene) {
    const sceneIndex = processScene(scene, this.#json, this.#processNode.bind(this));
    return sceneIndex;
  }

  #processNode(object: Object3D) {
    const cached = this.#cache.nodes.get(object);
    if (cached !== undefined) return cached;

    const index = processNode(
      object,
      this.#json,
      this.#skins,
      this.#processMesh.bind(this),
      this.#processNode.bind(this)
    );

    this.#cache.nodes.set(object, index);
    return index;
  }

  #processMesh(mesh: Mesh | Line | Points) {
    const cached = this.#cache.meshes.get(mesh);
    if (cached !== undefined) return cached;

    const index = processMesh(
      mesh,
      this.#json,
      this.#processAccessor.bind(this),
      this.#processMaterial.bind(this)
    );

    this.#cache.meshes.set(mesh, index);
    return index;
  }

  #processMaterial(material: Material) {
    const cached = this.#cache.materials.get(material);
    if (cached !== undefined) return cached;

    const index = processMaterial(material, this.#json, this.#processTexture.bind(this));

    this.#cache.materials.set(material, index);
    return index;
  }

  #processTexture(map: Texture) {
    const cached = this.#cache.textures.get(map);
    if (cached !== undefined) return cached;

    const index = processTexture(
      map,
      this.#json,
      this.#pending,
      this.#processSampler.bind(this),
      this.#processImage.bind(this)
    );

    this.#cache.textures.set(map, index);
    return index;
  }

  #processSampler(map: Texture) {
    const index = processSampler(map, this.#json);
    return index;
  }

  async #processImage(image: ImageBitmap, format: number, flipY: boolean, mimeType = "image/png") {
    const cachedImage = this.#cache.images.get(image);
    const cachedFlipY = cachedImage?.get(flipY);
    const cachedMimeType = cachedFlipY?.get(mimeType);
    if (cachedMimeType !== undefined) return cachedMimeType;

    const index = await processImage(
      image,
      format,
      flipY,
      mimeType,
      this.#json,
      this.#options,
      this.#processBufferViewImage.bind(this)
    );

    // Each image is cached according to its bitmap, flipY, and mimeType
    // So we use nested maps to cache each of these variations
    // ImageBitmap -> flipY -> mimeType -> index
    if (cachedImage === undefined) {
      const flipYMap = new Map();
      this.#cache.images.set(image, flipYMap);
    }

    if (cachedFlipY === undefined) {
      const mimeTypeMap = new Map();
      const flipYMap = this.#cache.images.get(image);
      flipYMap?.set(flipY, mimeTypeMap);
    }

    if (cachedMimeType === undefined) {
      const mimeTypeMap = this.#cache.images.get(image)?.get(flipY);
      mimeTypeMap?.set(mimeType, index);
    }

    return index;
  }

  #processAccessor(
    attribute: BufferAttribute | InterleavedBufferAttribute,
    geometry?: BufferGeometry,
    count?: number,
    start?: number
  ) {
    const index = processAccessor(
      attribute,
      this.#json,
      this.#processBufferView.bind(this),
      geometry,
      count,
      start
    );
    return index;
  }

  #processBufferView(
    attribute: BufferAttribute | InterleavedBufferAttribute,
    componentType: number,
    start: number,
    count: number
  ) {
    const { index, byteOffset } = processBufferView(
      attribute,
      componentType,
      start,
      count,
      this.#byteOffset,
      this.#json,
      this.#processBuffer.bind(this)
    );

    this.#byteOffset = byteOffset;
    return index;
  }

  async #processBufferViewImage(blob: Blob) {
    const { index, byteLength } = await processBufferViewImage(
      blob,
      this.#json,
      this.#byteOffset,
      this.#processBuffer.bind(this)
    );

    this.#byteOffset += byteLength;

    return index;
  }

  #processBuffer(buffer: ArrayBuffer) {
    if (!this.#json.buffers) this.#json.buffers = [{ byteLength: 0 }];
    // All buffers are merged before export
    this.#buffers.push(buffer);
    return 0;
  }
}
