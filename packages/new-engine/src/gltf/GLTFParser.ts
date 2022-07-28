import {
  AnimationClip,
  CanvasTexture,
  InterleavedBuffer,
  MeshStandardMaterial,
  Object3D,
} from "three";

import { BufferViewResult } from "./loader/loadBufferView";
import { buildNodeHierarchy } from "./parser/buildNodeHierarchy";
import { AccessorResult, loadAccessor } from "./parser/loadAccessor";
import { loadAnimation } from "./parser/loadAnimation";
import { loadMaterial } from "./parser/loadMaterial";
import { loadMesh } from "./parser/loadMesh";
import { loadNode } from "./parser/loadNode";
import { PrimitiveResult, loadPrimitive } from "./parser/loadPrimitive";
import { SceneResult, loadScene } from "./parser/loadScene";
import { SkinResult, loadSkin } from "./parser/loadSkin";
import { loadTexture } from "./parser/loadTexture";
import {
  GLTF,
  MaterialNormalTextureInfo,
  MaterialOcclusionTextureInfo,
  MeshPrimitive,
  TextureInfo,
} from "./schemaTypes";
import { LoadedGLTF } from "./types";

// Converts a loaded glTf to a three.js scene
export class GLTFParser {
  #json: GLTF;
  #bufferViews: BufferViewResult[];
  #images: ImageBitmap[];

  #scenes = new Map<number, Promise<SceneResult>>();

  // Cache
  #boneIndexes = new Set<number>();
  #skinnedMeshIndexes = new Set<number>();

  #meshReferenceCount = new Map<number, number>();
  #meshUseCount = new Map<number, number>();
  #nodeReferenceCount = new Map<number, number>();
  #nodeUseCount = new Map<number, number>();

  #accessors = new Map<number, Promise<AccessorResult>>();
  #animations = new Map<number, Promise<AnimationClip>>();
  #interleavedBuffers = new Map<string, InterleavedBuffer>();
  #materials = new Map<number, Promise<MeshStandardMaterial>>();
  #meshes = new Map<number, Promise<Object3D>>();
  #nodes = new Map<number, Promise<Object3D>>();
  #primitives = new Map<string, Promise<PrimitiveResult>>();
  #skins = new Map<number, Promise<SkinResult>>();
  #textures = new Map<string, Promise<CanvasTexture>>();

  constructor({ json, bufferViews = [], images = [] }: LoadedGLTF) {
    this.#json = json;
    this.#bufferViews = bufferViews;
    this.#images = images;
  }

  public async parse() {
    const sceneIndex = this.#json.scene ?? 0;
    const scene = await this.#loadScene(sceneIndex);
    return scene;
  }

  #loadScene(index: number) {
    const cached = this.#scenes.get(index);
    if (cached) return cached;

    // Clear cache
    this.#boneIndexes.clear();
    this.#skinnedMeshIndexes.clear();

    this.#meshReferenceCount.clear();
    this.#meshUseCount.clear();
    this.#nodeReferenceCount.clear();
    this.#nodeUseCount.clear();

    this.#accessors.clear();
    this.#animations.clear();
    this.#interleavedBuffers.clear();
    this.#materials.clear();
    this.#meshes.clear();
    this.#nodes.clear();
    this.#primitives.clear();
    this.#skins.clear();
    this.#textures.clear();

    const scene = loadScene(
      index,
      this.#json,
      this.#boneIndexes,
      this.#skinnedMeshIndexes,
      this.#meshReferenceCount,
      this.#buildNodeHierarchy.bind(this),
      this.#loadAnimation.bind(this)
    );

    this.#scenes.set(index, scene);
    return scene;
  }

  #buildNodeHierarchy(index: number) {
    const node = buildNodeHierarchy(
      index,
      this.#json,
      this.#loadNode.bind(this),
      this.#buildNodeHierarchy.bind(this)
    );

    return node;
  }

  #loadAnimation(index: number) {
    const cached = this.#animations.get(index);
    if (cached) return cached;

    const animation = loadAnimation(
      index,
      this.#json,
      this.#loadAccessor.bind(this),
      this.#loadNode.bind(this)
    );

    this.#animations.set(index, animation);
    return animation;
  }

  #loadAccessor(index: number) {
    const cached = this.#accessors.get(index);
    if (cached) return cached;

    const accessor = loadAccessor(index, this.#json, this.#bufferViews, this.#interleavedBuffers);

    this.#accessors.set(index, accessor);
    return accessor;
  }

  async #loadNode(index: number) {
    const cached = this.#nodes.get(index);
    if (cached) {
      const references = this.#nodeReferenceCount.get(index) ?? 0;
      if (references <= 1) return cached;

      const count = this.#nodeUseCount.get(index) ?? 0;
      this.#nodeUseCount.set(index, count + 1);

      const node = await cached;
      const clone = node.clone();
      clone.name = `${node.name}_instance_${count}`;
      return clone;
    }

    const node = loadNode(
      index,
      this.#json,
      this.#boneIndexes,
      this.#loadMesh.bind(this),
      this.#loadSkin.bind(this),
      this.#loadNode.bind(this)
    );

    this.#nodes.set(index, node);
    return node;
  }

  async #loadMesh(index: number) {
    const cache = this.#meshes.get(index);
    if (cache) {
      const references = this.#meshReferenceCount.get(index) ?? 0;
      if (references <= 1) return cache;

      const count = this.#meshUseCount.get(index) ?? 0;
      this.#meshUseCount.set(index, count + 1);

      const mesh = await cache;
      const clone = mesh.clone();
      clone.name = `${mesh.name}_instance_${count}`;
      return clone;
    }

    const mesh = loadMesh(
      index,
      this.#json,
      this.#skinnedMeshIndexes,
      this.#loadPrimitive.bind(this)
    );

    this.#meshes.set(index, mesh);
    return mesh;
  }

  #loadSkin(index: number) {
    const cached = this.#skins.get(index);
    if (cached) return cached;

    const skin = loadSkin(index, this.#json, this.#loadAccessor.bind(this));

    this.#skins.set(index, skin);
    return skin;
  }

  #loadPrimitive(primitiveDef: MeshPrimitive) {
    const cacheKey = JSON.stringify(primitiveDef);
    const cached = this.#primitives.get(cacheKey);
    if (cached) return cached;

    const primitive = loadPrimitive(
      primitiveDef,
      this.#loadAccessor.bind(this),
      this.#loadMaterial.bind(this)
    );

    this.#primitives.set(cacheKey, primitive);
    return primitive;
  }

  #loadMaterial(index: number) {
    const cached = this.#materials.get(index);
    if (cached) return cached;

    const material = loadMaterial(index, this.#json, this.#loadTexture.bind(this));

    this.#materials.set(index, material);
    return material;
  }

  #loadTexture(info: TextureInfo | MaterialNormalTextureInfo | MaterialOcclusionTextureInfo) {
    const cacheKey = JSON.stringify(info);
    const cached = this.#textures.get(cacheKey);
    if (cached) return cached;

    const texture = loadTexture(info, this.#json, this.#images);

    this.#textures.set(cacheKey, texture);
    return texture;
  }
}
