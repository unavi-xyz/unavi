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

// Converts a loaded glTf to Three.js objects
export class GLTFParser {
  private _json: GLTF;
  private _bufferViews: BufferViewResult[];
  private _images: ImageBitmap[];

  private _scenes = new Map<number, Promise<SceneResult>>();

  // Cache
  private _boneIndexes = new Set<number>();
  private _skinnedMeshIndexes = new Set<number>();

  private _meshReferenceCount = new Map<number, number>();
  private _meshUseCount = new Map<number, number>();
  private _nodeReferenceCount = new Map<number, number>();
  private _nodeUseCount = new Map<number, number>();

  private _accessors = new Map<number, Promise<AccessorResult>>();
  private _animations = new Map<number, Promise<AnimationClip>>();
  private _interleavedBuffers = new Map<string, InterleavedBuffer>();
  private _materials = new Map<number, Promise<MeshStandardMaterial>>();
  private _meshes = new Map<number, Promise<Object3D>>();
  private _nodes = new Map<number, Promise<Object3D>>();
  private _primitives = new Map<string, Promise<PrimitiveResult>>();
  private _skins = new Map<number, Promise<SkinResult>>();
  private _textures = new Map<string, Promise<CanvasTexture>>();

  constructor({ json, bufferViews = [], images = [] }: LoadedGLTF) {
    this._json = json;
    this._bufferViews = bufferViews;
    this._images = images;
  }

  public async parse() {
    const sceneIndex = this._json.scene ?? 0;
    const scene = await this._loadScene(sceneIndex);
    return scene;
  }

  private _loadScene(index: number) {
    const cached = this._scenes.get(index);
    if (cached) return cached;

    // Clear cache
    this._boneIndexes.clear();
    this._skinnedMeshIndexes.clear();

    this._meshReferenceCount.clear();
    this._meshUseCount.clear();
    this._nodeReferenceCount.clear();
    this._nodeUseCount.clear();

    this._accessors.clear();
    this._animations.clear();
    this._interleavedBuffers.clear();
    this._materials.clear();
    this._meshes.clear();
    this._nodes.clear();
    this._primitives.clear();
    this._skins.clear();
    this._textures.clear();

    const scene = loadScene(
      index,
      this._json,
      this._boneIndexes,
      this._skinnedMeshIndexes,
      this._meshReferenceCount,
      this._buildNodeHierarchy.bind(this),
      this._loadAnimation.bind(this)
    );

    this._scenes.set(index, scene);
    return scene;
  }

  private _buildNodeHierarchy(index: number) {
    const node = buildNodeHierarchy(
      index,
      this._json,
      this._loadNode.bind(this),
      this._buildNodeHierarchy.bind(this)
    );

    return node;
  }

  private _loadAnimation(index: number) {
    const cached = this._animations.get(index);
    if (cached) return cached;

    const animation = loadAnimation(
      index,
      this._json,
      this._loadAccessor.bind(this),
      this._loadNode.bind(this)
    );

    this._animations.set(index, animation);
    return animation;
  }

  private _loadAccessor(index: number) {
    const cached = this._accessors.get(index);
    if (cached) return cached;

    const accessor = loadAccessor(index, this._json, this._bufferViews, this._interleavedBuffers);

    this._accessors.set(index, accessor);
    return accessor;
  }

  private async _loadNode(index: number) {
    const cached = this._nodes.get(index);
    if (cached) {
      const references = this._nodeReferenceCount.get(index) ?? 0;
      if (references <= 1) return cached;

      const count = this._nodeUseCount.get(index) ?? 0;
      this._nodeUseCount.set(index, count + 1);

      const node = await cached;
      const clone = node.clone();
      clone.name = `${node.name}_instance_${count}`;
      return clone;
    }

    const node = loadNode(
      index,
      this._json,
      this._boneIndexes,
      this._loadMesh.bind(this),
      this._loadSkin.bind(this),
      this._loadNode.bind(this)
    );

    this._nodes.set(index, node);
    return node;
  }

  private async _loadMesh(index: number) {
    const cache = this._meshes.get(index);
    if (cache) {
      const references = this._meshReferenceCount.get(index) ?? 0;
      if (references <= 1) return cache;

      const count = this._meshUseCount.get(index) ?? 0;
      this._meshUseCount.set(index, count + 1);

      const mesh = await cache;
      const clone = mesh.clone();
      clone.name = `${mesh.name}_instance_${count}`;
      return clone;
    }

    const mesh = loadMesh(
      index,
      this._json,
      this._skinnedMeshIndexes,
      this._loadPrimitive.bind(this)
    );

    this._meshes.set(index, mesh);
    return mesh;
  }

  private _loadSkin(index: number) {
    const cached = this._skins.get(index);
    if (cached) return cached;

    const skin = loadSkin(index, this._json, this._loadAccessor.bind(this));

    this._skins.set(index, skin);
    return skin;
  }

  private _loadPrimitive(primitiveDef: MeshPrimitive) {
    const cacheKey = JSON.stringify(primitiveDef);
    const cached = this._primitives.get(cacheKey);
    if (cached) return cached;

    const primitive = loadPrimitive(
      primitiveDef,
      this._loadAccessor.bind(this),
      this._loadMaterial.bind(this)
    );

    this._primitives.set(cacheKey, primitive);
    return primitive;
  }

  private _loadMaterial(index: number) {
    const cached = this._materials.get(index);
    if (cached) return cached;

    const material = loadMaterial(index, this._json, this._loadTexture.bind(this));

    this._materials.set(index, material);
    return material;
  }

  private _loadTexture(
    info: TextureInfo | MaterialNormalTextureInfo | MaterialOcclusionTextureInfo
  ) {
    const cacheKey = JSON.stringify(info);
    const cached = this._textures.get(cacheKey);
    if (cached) return cached;

    const texture = loadTexture(info, this._json, this._images);

    this._textures.set(cacheKey, texture);
    return texture;
  }
}
