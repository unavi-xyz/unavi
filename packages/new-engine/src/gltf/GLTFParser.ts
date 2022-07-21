import {
  AnimationClip,
  AnimationMixer,
  Bone,
  BufferAttribute,
  BufferGeometry,
  CanvasTexture,
  ClampToEdgeWrapping,
  DoubleSide,
  Group,
  InterleavedBuffer,
  InterleavedBufferAttribute,
  Line,
  LineLoop,
  LineSegments,
  LinearFilter,
  LinearMipMapLinearFilter,
  Matrix4,
  Mesh,
  MeshStandardMaterial,
  NumberKeyframeTrack,
  Object3D,
  Points,
  QuaternionKeyframeTrack,
  RepeatWrapping,
  Skeleton,
  SkinnedMesh,
  VectorKeyframeTrack,
  sRGBEncoding,
} from "three";

import {
  GLTFCubicSplineInterpolant,
  GLTFCubicSplineQuaternionInterpolant,
} from "./CubicSplineInterpolation";
import {
  ALPHA_MODES,
  ATTRIBUTES,
  AttributeName,
  ComponentType,
  INTERPOLATION,
  PATH_PROPERTIES,
  TypeSize,
  WEBGL_COMPONENT_TYPES,
  WEBGL_CONSTANTS,
  WEBGL_FILTERS,
  WEBGL_TYPE_SIZES,
  WEBGL_WRAPPINGS,
} from "./constants";
import {
  BufferView,
  GLTF,
  MaterialNormalTextureInfo,
  MaterialOcclusionTextureInfo,
  MeshPrimitive,
  Sampler,
  TextureInfo,
} from "./schemaTypes";
import { LoadedBufferView, LoadedGLTF } from "./types";

type SceneResult = {
  scene: Group;
  animations: AnimationMixer;
};
type PrimitiveResult = [MeshPrimitive, BufferGeometry, MeshStandardMaterial];

// Converts a loaded glTf to Three.js objects
export class GLTFParser {
  private _json: GLTF;
  private _bufferViews: LoadedBufferView[];
  private _images: ImageBitmap[];

  private _scenes = new Map<number, SceneResult>();

  // Cache
  private _accessors = new Map<number, BufferAttribute | InterleavedBufferAttribute>();
  private _boneIndexes = new Set<number>();
  private _interleavedBuffers = new Map<string, InterleavedBuffer>();
  private _meshes = new Map<number, Object3D>();
  private _nodes = new Map<number, Object3D>();
  private _primitives = new Map<string, PrimitiveResult>();
  private _skinnedMeshIndexes = new Set<number>();
  private _textures = new Map<
    TextureInfo | MaterialNormalTextureInfo | MaterialOcclusionTextureInfo,
    CanvasTexture
  >();

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

  private async _loadScene(index: number) {
    const cached = this._scenes.get(index);
    if (cached !== undefined) return cached;

    if (!this._json.scenes) {
      throw new Error("No scenes found");
    }

    // Clear cache
    this._accessors.clear();
    this._boneIndexes.clear();
    this._interleavedBuffers.clear();
    this._meshes.clear();
    this._nodes.clear();
    this._primitives.clear();
    this._skinnedMeshIndexes.clear();
    this._textures.clear();

    const sceneDef = this._json.scenes[index];
    const scene = new Group();
    scene.name = sceneDef.name ?? `scene_${index}`;

    // Mark bones
    if (this._json.skins !== undefined) {
      this._json.skins.forEach((skin) => {
        skin.joints.forEach((jointIndex) => {
          this._boneIndexes.add(jointIndex);
        });
      });
    }

    // Mark skinned meshes
    sceneDef.nodes?.forEach(async (nodeIndex) => {
      if (!this._json.nodes) {
        throw new Error("No nodes found");
      }

      const nodeDef = this._json.nodes[nodeIndex];
      if (nodeDef.skin !== undefined) {
        if (nodeDef.skin !== undefined) {
          this._skinnedMeshIndexes.add(nodeIndex);
        }
      }
    });

    // Load Nodes
    const nodePromises = sceneDef.nodes?.map(async (nodeIndex) => {
      const node = await this._loadNode(nodeIndex);
      scene.add(node);
    });
    await Promise.all(nodePromises ?? []);

    // Load Animations
    const animationPromises = this._json.animations?.map(async (_, index) => {
      const animation = await this._loadAnimation(index);
      return animation;
    });
    const animationClips = await Promise.all(animationPromises ?? []);

    const animationMixer = new AnimationMixer(scene);
    const animationActions = animationClips.map((clip) => animationMixer.clipAction(clip));

    animationActions.forEach((action) => {
      action.play();
    });

    const result: SceneResult = {
      scene,
      animations: animationMixer,
    };
    this._scenes.set(index, result);
    return result;
  }

  private async _loadAnimation(index: number) {
    if (this._json.animations === undefined) {
      throw new Error("No animations found");
    }

    const animationDef = this._json.animations[index];

    // Load channels
    const channelPromises = animationDef.channels.map(async (channel) => {
      const sampler = animationDef.samplers[channel.sampler];

      const input = await this._loadAccessor(sampler.input);
      const output = await this._loadAccessor(sampler.output);

      if (channel.target.node === undefined) {
        throw new Error(`Animation target has no node`);
      }

      if (input === null) {
        throw new Error(`Animation input is null`);
      }

      if (output === null) {
        throw new Error(`Animation output is null`);
      }

      const node = await this._loadNode(channel.target.node);

      node.updateMatrix();
      node.matrixAutoUpdate = true;

      const interpolationType = sampler.interpolation ?? "LINEAR";
      const interpolation = INTERPOLATION[interpolationType];

      let TypedKeyframeTrack:
        | typeof NumberKeyframeTrack
        | typeof QuaternionKeyframeTrack
        | typeof VectorKeyframeTrack;

      switch (channel.target.path) {
        case "weights":
          TypedKeyframeTrack = NumberKeyframeTrack;
          break;
        case "rotation":
          TypedKeyframeTrack = QuaternionKeyframeTrack;
          break;
        case "translation":
        case "scale":
        default:
          TypedKeyframeTrack = VectorKeyframeTrack;
      }

      const names: string[] = [];

      if (channel.target.path === "weights") {
        node.traverse((child) => {
          if ("morphTargetInfluences" in child) {
            names.push(child.uuid);
          }
        });
      } else {
        names.push(node.uuid);
      }

      const channelTracks = names.map((name) => {
        const track = new TypedKeyframeTrack(
          `${name}.${PATH_PROPERTIES[channel.target.path]}`,
          Array.from(input.array),
          Array.from(output.array),
          interpolation
        );

        // Override interpolation with custom factory method
        // The built in three.js cubic interpolation does not work with glTF
        if (sampler.interpolation === "CUBICSPLINE") {
          // @ts-ignore
          track.createInterpolant = function InterpolantFactoryMethodGLTFCubicSpline(result) {
            // A CUBICSPLINE keyframe in glTF has three output values for each input value,
            // representing inTangent, splineVertex, and outTangent. As a result, track.getValueSize()
            // must be divided by three to get the interpolant's sampleSize argument.

            const interpolantType =
              this instanceof QuaternionKeyframeTrack
                ? GLTFCubicSplineQuaternionInterpolant
                : GLTFCubicSplineInterpolant;

            return new interpolantType(this.times, this.values, this.getValueSize() / 3, result);
          };

          // Mark as CUBICSPLINE. `track.getInterpolation()` doesn't support custom interpolants.
          // @ts-ignore
          track.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline = true;
        }

        return track;
      });

      return channelTracks;
    });
    const tracks = await Promise.all(channelPromises);
    const flattened = tracks.flat();

    // Create animation
    const name = animationDef.name ?? `animation_${index}`;
    const animationClip = new AnimationClip(name, undefined, flattened);

    return animationClip;
  }

  private async _loadNode(index: number) {
    const cached = this._nodes.get(index);
    if (cached) return cached;

    if (this._json.nodes === undefined) {
      throw new Error("No nodes found");
    }

    const nodeDef = this._json.nodes[index];
    const node = this._boneIndexes.has(index) ? new Bone() : new Group();

    // Transform
    if (nodeDef.matrix !== undefined) {
      const matrix = new Matrix4();
      matrix.fromArray(nodeDef.matrix);
      node.applyMatrix4(matrix);
    } else {
      if (nodeDef.translation) {
        node.position.fromArray(nodeDef.translation);
      }

      if (nodeDef.rotation) {
        node.quaternion.fromArray(nodeDef.rotation);
      }

      if (nodeDef.scale) {
        node.scale.fromArray(nodeDef.scale);
      }
    }

    // Mesh
    if (nodeDef.mesh !== undefined) {
      const mesh = await this._loadMesh(nodeDef.mesh);
      node.add(mesh);
    }

    // Skin
    if (nodeDef.skin !== undefined) {
      const skinEntry = await this._loadSkin(nodeDef.skin);

      const jointPromises = skinEntry.joints.map(async (joint) => {
        return await this._loadNode(joint);
      });
      const joints = await Promise.all(jointPromises);

      node.traverse((child: Object3D | SkinnedMesh) => {
        if (!(child instanceof SkinnedMesh)) return;

        const bones: Bone[] = [];
        const boneInverses: Matrix4[] = [];

        joints.forEach((joint, i) => {
          if (!(joint instanceof Bone)) return;

          const inverseMatrix = new Matrix4();

          if (skinEntry.inverseBindMatrices !== null) {
            inverseMatrix.fromArray(skinEntry.inverseBindMatrices.array, i * 16);
          }

          bones.push(joint);
          boneInverses.push(inverseMatrix);
        });

        child.bind(new Skeleton(bones, boneInverses), child.matrixWorld);
      });
    }

    // Children
    if (nodeDef.children) {
      const childrenPromises = nodeDef.children.map(async (childIndex) => {
        const child = await this._loadNode(childIndex);
        return child;
      });
      const children = await Promise.all(childrenPromises);

      children.forEach((child) => {
        node.add(child);
      });
    }

    this._nodes.set(index, node);
    return node;
  }

  private async _loadSkin(index: number) {
    if (this._json.skins === undefined) {
      throw new Error("No skins found");
    }

    const skinDef = this._json.skins[index];

    if (skinDef.inverseBindMatrices === undefined) {
      throw new Error("No meshes found");
    }

    const accessor = await this._loadAccessor(skinDef.inverseBindMatrices);

    const skinEntry = {
      joints: skinDef.joints,
      inverseBindMatrices: accessor,
    };

    return skinEntry;
  }

  private async _loadMesh(index: number) {
    const cached = this._meshes.get(index);
    if (cached) return cached;

    if (this._json.meshes === undefined) {
      throw new Error("No meshes found");
    }

    const meshDef = this._json.meshes[index];

    // Load geometry
    const primitivePromises = meshDef.primitives.map((primitive) => this._loadPrimitive(primitive));
    const primitives = await Promise.all(primitivePromises);

    // Create meshes
    const meshes = primitives.map(([primitive, geometry, material], i) => {
      let mesh: Mesh | Line | Points;

      switch (primitive.mode) {
        case WEBGL_CONSTANTS.TRIANGLES:
        case WEBGL_CONSTANTS.TRIANGLE_STRIP:
        case WEBGL_CONSTANTS.TRIANGLE_FAN:
        case undefined:
          const isSkinnedMesh = this._skinnedMeshIndexes.has(index);
          mesh = isSkinnedMesh ? new SkinnedMesh(geometry, material) : new Mesh(geometry, material);

          if (mesh instanceof SkinnedMesh && !mesh.geometry.attributes.skinWeight.normalized) {
            mesh.normalizeSkinWeights();
          }

          break;
        case WEBGL_CONSTANTS.LINES:
          mesh = new LineSegments(geometry, material);
          break;
        case WEBGL_CONSTANTS.LINE_STRIP:
          mesh = new Line(geometry, material);
          break;
        case WEBGL_CONSTANTS.LINE_LOOP:
          mesh = new LineLoop(geometry, material);
          break;
        case WEBGL_CONSTANTS.POINTS:
          mesh = new Points(geometry, material);
          break;
        default:
          throw new Error(`Unknown primitive mode ${primitive.mode}`);
      }

      mesh.name = meshDef.name ?? `mesh_${index}_${i}`;
      return mesh;
    });

    // Set morph targets
    meshes.forEach((mesh) => {
      if (Object.keys(mesh.geometry.morphAttributes).length > 0) {
        mesh.updateMorphTargets();

        if (meshDef.weights !== undefined) {
          mesh.morphTargetInfluences = [...meshDef.weights];
        }
      }
    });

    const mesh = meshes.length === 1 ? meshes[0] : new Group().add(...meshes);
    this._meshes.set(index, mesh);
    return mesh;
  }

  private async _loadPrimitive(primitiveDef: MeshPrimitive) {
    const cacheKey = JSON.stringify(primitiveDef);
    const cached = this._primitives.get(cacheKey);
    if (cached) return cached;

    const geometry = new BufferGeometry();

    // Attributes
    const attributePromises = Object.entries(primitiveDef.attributes).map(
      async ([name, accessorId]) => {
        const threeName = ATTRIBUTES[name as AttributeName] ?? name.toLowerCase();
        if (threeName in geometry.attributes) return;

        const attribute = await this._loadAccessor(accessorId);

        if (attribute === null) {
          throw new Error(`Attribute ${name} not found`);
        }

        geometry.setAttribute(threeName, attribute);
      }
    );
    await Promise.all(attributePromises);

    // Indices
    if (primitiveDef.indices !== undefined && geometry.index == null) {
      const accessor = await this._loadAccessor(primitiveDef.indices);

      if (accessor === null) {
        throw new Error(`Indices not found`);
      }

      if (accessor instanceof InterleavedBufferAttribute) {
        throw new Error(`Indices are interleaved`);
      }

      geometry.setIndex(accessor);
    }

    // Material
    const material =
      primitiveDef.material === undefined
        ? new MeshStandardMaterial()
        : await this._loadMaterial(primitiveDef.material);

    // Occlusion map needs a second set of UVs
    if (
      material.aoMap &&
      geometry.attributes.uv2 === undefined &&
      geometry.attributes.uv !== undefined
    ) {
      geometry.setAttribute("uv2", geometry.attributes.uv);
    }

    // Enable flat shading
    if (geometry.attributes.normal === undefined) {
      material.flatShading = true;
    }

    // Enable vertex colors
    if (geometry.attributes.color !== undefined) {
      material.vertexColors = true;
    }

    // If three.js needs to generate tangents, flip normal map y
    // https://github.com/mrdoob/three.js/issues/11438#issuecomment-507003995
    if (geometry.attributes.tangent === undefined) {
      material.normalScale.y *= -1;
    }

    // Morph targets
    if (primitiveDef.targets !== undefined) {
      const targetPromises = primitiveDef.targets.map(async (target) => {
        const entryPromises = Object.entries(target).map(async ([name, accessorId]) => {
          const accessor = await this._loadAccessor(accessorId);

          if (accessor === null) {
            throw new Error(`Target ${name} not found`);
          }

          switch (name) {
            case "POSITION":
              if (geometry.morphAttributes.position === undefined) {
                geometry.morphAttributes.position = [];
              }

              geometry.morphAttributes.position.push(accessor);
              break;
            case "NORMAL":
              if (geometry.morphAttributes.normal === undefined) {
                geometry.morphAttributes.normal = [];
              }

              geometry.morphAttributes.normal.push(accessor);
              break;
            case "TANGENT":
              if (geometry.morphAttributes.tangent === undefined) {
                geometry.morphAttributes.tangent = [];
              }

              geometry.morphAttributes.tangent.push(accessor);
              break;
            case "COLOR_0":
              if (geometry.morphAttributes.color === undefined) {
                geometry.morphAttributes.color = [];
              }

              geometry.morphAttributes.color.push(accessor);
              break;
          }
        });
        await Promise.all(entryPromises);
      });
      await Promise.all(targetPromises);

      geometry.morphTargetsRelative = true;
    }

    const result: PrimitiveResult = [primitiveDef, geometry, material];
    this._primitives.set(cacheKey, result);
    return result;
  }

  private async _loadMaterial(index: number) {
    if (this._json.materials === undefined) {
      throw new Error("No materials found");
    }

    const materialDef = this._json.materials[index];
    const material = new MeshStandardMaterial();
    material.name = materialDef.name ?? `material_${index}`;

    const {
      baseColorFactor = [1, 1, 1, 1],
      metallicFactor = 1,
      roughnessFactor = 1,
      baseColorTexture,
      metallicRoughnessTexture,
    } = materialDef.pbrMetallicRoughness ?? {};

    material.color.fromArray(baseColorFactor);
    material.opacity = baseColorFactor[3];
    material.metalness = metallicFactor;
    material.roughness = roughnessFactor;

    if (materialDef.doubleSided) material.side = DoubleSide;

    const emissiveFactor = materialDef.emissiveFactor ?? [0, 0, 0];
    material.emissive.fromArray(emissiveFactor);

    // Alpha
    const alphaMode = materialDef.alphaMode ?? ALPHA_MODES.OPAQUE;

    if (alphaMode === ALPHA_MODES.BLEND) {
      material.transparent = true;
      material.depthWrite = false;
    } else {
      material.transparent = false;

      if (alphaMode === ALPHA_MODES.MASK) {
        material.alphaTest = materialDef.alphaCutoff ?? 0.5;
      }
    }

    // Textures
    if (baseColorTexture) {
      const texture = await this._loadTexture(baseColorTexture);
      texture.encoding = sRGBEncoding;
      material.map = texture;
    }

    if (metallicRoughnessTexture) {
      const texture = await this._loadTexture(metallicRoughnessTexture);
      material.metalnessMap = texture;
      material.roughnessMap = texture;
    }

    if (materialDef.normalTexture) {
      const texture = await this._loadTexture(materialDef.normalTexture);
      material.normalMap = texture;
      const scale = materialDef.normalTexture.scale ?? 1;
      material.normalScale.set(scale, scale);
    }

    if (materialDef.occlusionTexture) {
      const texture = await this._loadTexture(materialDef.occlusionTexture);
      material.aoMap = texture;
      material.aoMapIntensity = materialDef.occlusionTexture.strength ?? 1;
    }

    if (materialDef.emissiveTexture) {
      const texture = await this._loadTexture(materialDef.emissiveTexture);
      texture.encoding = sRGBEncoding;
      material.emissiveMap = texture;
    }

    return material;
  }

  private async _loadTexture(
    info: TextureInfo | MaterialNormalTextureInfo | MaterialOcclusionTextureInfo
  ) {
    const cached = this._textures.get(info);
    if (cached) return cached;

    if (this._json.textures === undefined) {
      throw new Error("No textures found");
    }

    const textureDef = this._json.textures[info.index];

    if (textureDef.source === undefined) {
      throw new Error(`Texture ${info.index} has no source`);
    }

    // Create texture
    const image = this._images[textureDef.source];
    const texture = new CanvasTexture(image);
    texture.needsUpdate = true;
    texture.flipY = false;
    if (textureDef.name) texture.name = textureDef.name;

    // Sampler
    let samplerDef: Sampler = {};
    const samplerIndex = textureDef.sampler;
    if (samplerIndex !== undefined) {
      if (this._json.samplers === undefined) {
        throw new Error("No samplers found");
      }
      samplerDef = this._json.samplers[samplerIndex];
    }

    const magFilter = samplerDef.magFilter as keyof typeof WEBGL_FILTERS;
    texture.magFilter = WEBGL_FILTERS[magFilter] ?? LinearFilter;

    const minFilter = samplerDef.minFilter as keyof typeof WEBGL_FILTERS;
    texture.minFilter = WEBGL_FILTERS[minFilter] ?? LinearMipMapLinearFilter;

    const wrapS = samplerDef.wrapS as keyof typeof WEBGL_WRAPPINGS;
    texture.wrapS = WEBGL_WRAPPINGS[wrapS] ?? RepeatWrapping;

    const wrapT = samplerDef.wrapT as keyof typeof WEBGL_WRAPPINGS;
    texture.wrapT = WEBGL_WRAPPINGS[wrapT] ?? RepeatWrapping;

    this._textures.set(info, texture);
    return texture;
  }

  private async _loadAccessor(index: number) {
    const cached = this._accessors.get(index);
    if (cached) return cached;

    if (this._json.accessors === undefined) {
      throw new Error("No accessors found");
    }

    const accessorDef = this._json.accessors[index];

    if (accessorDef.bufferView === undefined && accessorDef.sparse === undefined) {
      // Ignore empty accessors, which may be used to declare runtime
      // information about attributes coming from another source (e.g. Draco
      // compression extension).
      return null;
    }

    // Load buffer views
    const views: ({ bufferViewDef: BufferView; bufferView: ArrayBuffer } | null)[] = [];

    if (accessorDef.bufferView !== undefined) {
      const bufferView = await this._bufferViews[accessorDef.bufferView];
      views.push(bufferView);
    } else {
      views.push(null);
    }

    if (accessorDef.sparse !== undefined) {
      const indices = await this._bufferViews[accessorDef.sparse.indices.bufferView];
      const values = await this._bufferViews[accessorDef.sparse.values.bufferView];

      views.push(indices, values);
    }

    if (accessorDef.componentType in WEBGL_COMPONENT_TYPES === false) {
      throw new Error(`Invalid component type: ${accessorDef.componentType}`);
    }

    const componentType = accessorDef.componentType as ComponentType;
    const TypedArray = WEBGL_COMPONENT_TYPES[componentType];

    if (accessorDef.type in WEBGL_TYPE_SIZES === false) {
      throw new Error(`Invalid type: ${accessorDef.type}`);
    }

    const { bufferViewDef, bufferView } = views[0]
      ? views[0]
      : { bufferViewDef: null, bufferView: null };

    const type = accessorDef.type as TypeSize;
    const itemSize = WEBGL_TYPE_SIZES[type];

    const elementBytes = TypedArray.BYTES_PER_ELEMENT;
    const itemBytes = elementBytes * itemSize;
    const byteStride = bufferViewDef?.byteStride;
    const byteOffset = accessorDef.byteOffset ?? 0;
    const normalized = accessorDef.normalized;

    // Create buffer attribute
    let bufferAttribute: BufferAttribute | InterleavedBufferAttribute;

    const isInterleaved = byteStride && byteStride !== itemBytes && bufferView;
    if (isInterleaved) {
      const ibSlice = Math.floor(byteOffset / byteStride);
      const cacheKey = `${accessorDef.bufferView}-${accessorDef.componentType}-${ibSlice}-${accessorDef.count}`;
      let interleavedBuffer = this._interleavedBuffers.get(cacheKey);

      if (interleavedBuffer === undefined) {
        const array = new TypedArray(
          bufferView,
          ibSlice * byteStride,
          (accessorDef.count * byteStride) / elementBytes
        );

        interleavedBuffer = new InterleavedBuffer(array, byteStride / elementBytes);

        this._interleavedBuffers.set(cacheKey, interleavedBuffer);
      }

      if (interleavedBuffer === undefined) {
        throw new Error("Failed to create interleaved buffer");
      }

      bufferAttribute = new InterleavedBufferAttribute(
        interleavedBuffer,
        itemSize,
        (byteOffset % byteStride) / elementBytes,
        normalized
      );
    } else {
      let array: ArrayLike<number>;

      if (bufferView === null) {
        array = new TypedArray(accessorDef.count * itemSize);
      } else {
        array = new TypedArray(bufferView, byteOffset, accessorDef.count * itemSize);
      }

      bufferAttribute = new BufferAttribute(array, itemSize, normalized);
    }

    // Sparse
    if (accessorDef.sparse !== undefined && views[1] !== null && views[2] !== null) {
      const itemSizeIndices = WEBGL_TYPE_SIZES.SCALAR;
      const indicesComponentType = accessorDef.sparse.indices.componentType as ComponentType;

      if (indicesComponentType in WEBGL_COMPONENT_TYPES === false) {
        throw new Error(`Invalid component type: ${accessorDef.componentType}`);
      }

      const TypedArrayIndices = WEBGL_COMPONENT_TYPES[indicesComponentType];

      const byteOffsetIndices = accessorDef.sparse.indices.byteOffset ?? 0;
      const byteOffsetValues = accessorDef.sparse.values.byteOffset ?? 0;

      const sparseIndices = new TypedArrayIndices(
        views[1].bufferView,
        byteOffsetIndices,
        accessorDef.sparse.count * itemSizeIndices
      );
      const sparseValues = new TypedArray(
        views[2].bufferView,
        byteOffsetValues,
        accessorDef.sparse.count * itemSize
      );

      if (bufferView !== null) {
        // Avoid modifying the original ArrayBuffer, if the bufferView wasn't initialized with zeroes.
        bufferAttribute = new BufferAttribute(
          new TypedArray(bufferAttribute.array),
          itemSize,
          normalized
        );
      }

      sparseIndices.forEach((index, i) => {
        bufferAttribute.setX(index, sparseValues[i * itemSize]);
        if (itemSize >= 2) bufferAttribute.setY(index, sparseValues[i * itemSize + 1]);
        if (itemSize >= 3) bufferAttribute.setZ(index, sparseValues[i * itemSize + 2]);
        if (itemSize >= 4) bufferAttribute.setW(index, sparseValues[i * itemSize + 3]);
        if (itemSize >= 5) throw new Error("Unsupported itemSize in sparse BufferAttribute.");
      });
    }

    bufferAttribute.name = accessorDef.name ?? `bufferAttribute_${index}`;
    this._accessors.set(index, bufferAttribute);
    return bufferAttribute;
  }
}
