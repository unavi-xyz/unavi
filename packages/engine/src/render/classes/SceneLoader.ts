import {
  ComponentType,
  DESERIALIZE_MODE,
  TypedArray,
  createWorld,
  defineDeserializer,
  defineQuery,
} from "bitecs";
import {
  AnimationClip,
  Bone,
  BufferAttribute,
  BufferGeometry,
  CanvasTexture,
  ClampToEdgeWrapping,
  DoubleSide,
  FrontSide,
  Group,
  InterpolateDiscrete,
  InterpolateLinear,
  InterpolationModes,
  Line,
  LineLoop,
  LineSegments,
  LinearFilter,
  LinearMipMapLinearFilter,
  LinearMipMapNearestFilter,
  Matrix4,
  Mesh,
  MeshStandardMaterial,
  MirroredRepeatWrapping,
  NearestFilter,
  NearestMipMapLinearFilter,
  NearestMipMapNearestFilter,
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
  AlphaMode,
  AnimationChannel,
  AnimationSampler,
  AttributeColor,
  AttributeNormal,
  AttributePosition,
  AttributeSkinIndex,
  AttributeSkinWeight,
  AttributeTangent,
  AttributeType,
  AttributeUV,
  AttributeUV2,
  ColorTexture,
  EmissiveTexture,
  Interpolation,
  Material,
  MetallicRoughnessTexture,
  MorphTarget,
  Node,
  NodeMesh,
  NodeParent,
  NodeSkin,
  NormalTexture,
  OcclusionTexture,
  Primitive,
  PrimitiveIndices,
  PrimitiveMaterial,
  Skin,
  SkinJoint,
  TargetPath,
  Texture,
  TextureInfo,
  animationChannelQuery,
  animationQuery,
  config,
  jointQuery,
  materialQuery,
  materialWithColorTextureQuery,
  materialWithEmissiveTextureQuery,
  materialWithMetallicRoughnessTextureQuery,
  materialWithNormalTextureQuery,
  materialWithOcclusionTextureQuery,
  nodeQuery,
  nodeWithMeshAndSkinQuery,
  nodeWithMeshQuery,
  nodeWithParentQuery,
  nodeWithSkinQuery,
  nodeWithoutParentQuery,
  primitiveQuery,
  primitiveWithIndicesQuery,
} from "../../ecs";
import { ThreeAnimationPathName as ThreePathName, WEBGL_CONSTANTS } from "../constants";
import { LoadSceneData } from "../types";
import {
  GLTFCubicSplineInterpolant,
  GLTFCubicSplineQuaternionInterpolant,
} from "./CubicSplineInterpolation";

type PrimitiveObject3D = Mesh | Line | LineSegments | LineLoop | Points;
type KeyframeTrack = NumberKeyframeTrack | QuaternionKeyframeTrack | VectorKeyframeTrack;
type KeyframeTrackConstructor =
  | typeof NumberKeyframeTrack
  | typeof QuaternionKeyframeTrack
  | typeof VectorKeyframeTrack;

// Loads an ECS world into a three.js scene
export class SceneLoader {
  #world = createWorld(config);
  #images: ImageBitmap[] = [];
  #accessors: TypedArray[] = [];

  #scene = new Group();
  #animations: AnimationClip[] = [];

  #materials = new Map<number, MeshStandardMaterial>();
  #geometies = new Map<number, BufferGeometry>();
  #primitives = new Map<number, PrimitiveObject3D>();
  #meshes = new Map<number, number[]>();
  #objects = new Map<number, Object3D>();

  #deserialize = defineDeserializer(config);

  parse({ worldBuffer, images, accessors }: LoadSceneData) {
    this.#images = images;
    this.#accessors = accessors;
    this.#world = createWorld(config);
    this.#deserialize(this.#world, worldBuffer, DESERIALIZE_MODE.MAP);

    // Parse materials
    this.#parseMaterials();

    // Parse primitives
    this.#parsePrimitives();

    // Parse nodes
    this.#parseNodes();

    // Parse animations
    this.#parseAnimations();

    return { scene: this.#scene, animations: this.#animations };
  }

  #parseAnimations() {
    // Create animation clips
    const animations = animationQuery(this.#world);
    const animationChannels = animationChannelQuery(this.#world);
    const clips = animations.map((aeid) => {
      let tracks: KeyframeTrack[] = [];

      animationChannels.forEach((eid) => {
        const animation = AnimationChannel.animation[eid];
        if (animation !== aeid) return;

        // Create keyframe track
        const sampler = AnimationChannel.sampler[eid];
        const input = AnimationSampler.input[sampler];
        const output = AnimationSampler.output[sampler];
        const inputArray = Array.from(this.#accessors[input]);
        const outputArray = Array.from(this.#accessors[output]);

        const targetPath = AnimationChannel.targetPath[eid];
        let threePath: string;
        let TypedKeyframeTrack: KeyframeTrackConstructor;
        switch (targetPath) {
          case TargetPath.weights:
            TypedKeyframeTrack = NumberKeyframeTrack;
            threePath = ThreePathName.weights;
            break;
          case TargetPath.rotation:
            TypedKeyframeTrack = QuaternionKeyframeTrack;
            threePath = ThreePathName.rotation;
            break;
          case TargetPath.translation:
            threePath = ThreePathName.translation;
            TypedKeyframeTrack = VectorKeyframeTrack;
            break;
          case TargetPath.scale:
            threePath = ThreePathName.scale;
            TypedKeyframeTrack = VectorKeyframeTrack;
            break;
          default:
            throw new Error(`Unknown target path: ${targetPath}`);
        }

        const interpolation = AnimationSampler.interpolation[sampler];
        let interpolationMode: InterpolationModes | undefined;
        switch (interpolation) {
          case Interpolation.LINEAR:
            interpolationMode = InterpolateLinear;
            break;
          case Interpolation.STEP:
            interpolationMode = InterpolateDiscrete;
            break;
          case Interpolation.CUBICSPLINE:
            interpolationMode = undefined;
            break;
          default:
            throw new Error(`Unknown interpolation: ${interpolation}`);
        }

        const target = AnimationChannel.target[eid];
        const node = this.#objects.get(target);
        if (!node) throw new Error("Node not found");

        const names = [];
        if (targetPath === TargetPath.weights) {
          node.traverse((child) => {
            if ("morphTargetInfluences" in child) names.push(child.uuid);
          });
        } else {
          names.push(node.uuid);
        }

        names.forEach((name) => {
          const track = new TypedKeyframeTrack(
            `${name}.${threePath}`,
            inputArray,
            outputArray,
            interpolationMode
          );

          if (interpolation === Interpolation.CUBICSPLINE) {
            // Create a custom interpolant for cubic spline interpolation
            // The built in three.js interpolant is not compatible with the glTF spec
            // @ts-ignore
            track.createInterpolant = function InterpolantFactoryMethodGLTFCubicSpline(result) {
              // A CUBICSPLINE keyframe in glTF has three output values for each input value,
              // representing inTangent, splineVertex, and outTangent. As a result, track.getValueSize()
              // must be divided by three to get the interpolant's sampleSize argument.
              const InterpolantType =
                this instanceof QuaternionKeyframeTrack
                  ? GLTFCubicSplineQuaternionInterpolant
                  : GLTFCubicSplineInterpolant;

              return new InterpolantType(this.times, this.values, this.getValueSize() / 3, result);
            };
            // @ts-ignore
            track.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline = true;
          }

          tracks.push(track);
        });
      });

      const clip = new AnimationClip("", undefined, tracks);
      return clip;
    });

    this.#animations = clips;
  }

  #parseNodes() {
    // Create nodes
    const nodes = nodeQuery(this.#world);
    const joints = jointQuery(this.#world);
    nodes.forEach((eid) => {
      const isBone = joints.reduce((acc, jeid) => acc || SkinJoint.bone[jeid] === eid, false);
      const object = isBone ? new Bone() : new Object3D();
      this.#objects.set(eid, object);

      // Set transform
      object.position.x = Node.position.x[eid];
      object.position.y = Node.position.y[eid];
      object.position.z = Node.position.z[eid];

      object.quaternion.x = Node.rotation.x[eid];
      object.quaternion.y = Node.rotation.y[eid];
      object.quaternion.z = Node.rotation.z[eid];
      object.quaternion.w = Node.rotation.w[eid];

      object.scale.x = Node.scale.x[eid];
      object.scale.y = Node.scale.y[eid];
      object.scale.z = Node.scale.z[eid];
    });

    // Add meshes to nodes
    const withMesh = nodeWithMeshQuery(this.#world);
    withMesh.forEach((eid) => {
      const object = this.#objects.get(eid);
      if (!object) throw new Error("Node not found");

      const meid = NodeMesh.mesh[eid];
      const primitives = this.#meshes.get(meid);
      if (!primitives) throw new Error("Primitives not found");

      primitives.forEach((peid) => {
        const primitive = this.#primitives.get(peid);
        if (!primitive) throw new Error("Primitive not found");
        object.add(primitive);
      });
    });

    // Load skins
    const withSkin = nodeWithSkinQuery(this.#world);
    withSkin.forEach((eid) => {
      const object = this.#objects.get(eid);
      if (!object) throw new Error("Node not found");

      const skinId = NodeSkin.skin[eid];
      const index = Skin.inverseBindMatrices[skinId];
      const inverseBindMatrices = this.#accessors[index];

      object.traverse((child) => {
        if (!(child instanceof SkinnedMesh)) return;

        const bones: Bone[] = [];
        const boneInverses: Matrix4[] = [];

        joints.forEach((jeid, i) => {
          const beid = SkinJoint.bone[jeid];
          const bone = this.#objects.get(beid);
          if (!bone) throw new Error("Joint not found");
          if (!(bone instanceof Bone)) throw new Error("Joint is not a bone");

          const matrix = new Matrix4();
          matrix.fromArray(inverseBindMatrices, i * 16);

          bones.push(bone);
          boneInverses.push(matrix);
        });

        child.bind(new Skeleton(bones, boneInverses), child.matrixWorld);
      });
    });

    // Parent nodes
    const withParent = nodeWithParentQuery(this.#world);
    withParent.forEach((eid) => {
      const object = this.#objects.get(eid);
      if (!object) throw new Error("Node not found");
      const parentId = NodeParent.parent[eid];
      const parent = this.#objects.get(parentId);
      if (!parent) throw new Error("Parent not found");
      parent.add(object);
    });

    const withoutParent = nodeWithoutParentQuery(this.#world);
    withoutParent.forEach((eid) => {
      const object = this.#objects.get(eid);
      if (!object) throw new Error("Node not found");
      this.#scene.add(object);
    });
  }

  #parsePrimitives() {
    // Create primitives
    const primitives = primitiveQuery(this.#world);
    primitives.forEach((eid) => {
      // Create geometry
      const geometry = new BufferGeometry();
      geometry.morphTargetsRelative = true;
      this.#geometies.set(eid, geometry);

      // Add to mesh
      const meshId = Primitive.mesh[eid];
      const list = this.#meshes.get(meshId) ?? [];
      list.push(eid);
      this.#meshes.set(meshId, list);
    });

    // Set indices
    const withIndicies = primitiveWithIndicesQuery(this.#world);
    withIndicies.forEach((eid) => {
      const indices = this.#parseAttribute(PrimitiveIndices, eid);
      const geometry = this.#geometies.get(eid);
      if (!geometry) throw new Error("Geometry not found");
      geometry.setIndex(indices);
    });

    // Set attributes
    this.#setPrimitiveAttributes("position", AttributePosition);
    this.#setPrimitiveAttributes("normal", AttributeNormal);
    this.#setPrimitiveAttributes("tangent", AttributeTangent);
    this.#setPrimitiveAttributes("color", AttributeColor);
    this.#setPrimitiveAttributes("uv", AttributeUV);
    this.#setPrimitiveAttributes("uv2", AttributeUV2);
    this.#setPrimitiveAttributes("skinIndex", AttributeSkinIndex);
    this.#setPrimitiveAttributes("skinWeight", AttributeSkinWeight);

    // Create meshes
    primitives.forEach((eid) => {
      const geometry = this.#geometies.get(eid);
      if (!geometry) throw new Error("No geometry found");

      const meid = PrimitiveMaterial.material[eid];
      const material = this.#materials.get(meid) ?? new MeshStandardMaterial();

      // Occlusion map needs a second set of UVs
      if (material.aoMap && geometry.attributes.uv && !geometry.attributes.uv2) {
        geometry.setAttribute("uv2", geometry.attributes.uv);
      }

      // Enable flat shading if no normal attribute
      if (!geometry.attributes.normal) material.flatShading = true;

      // Enable vertex colors if color attribute
      if (geometry.attributes.color) material.vertexColors = true;

      // If three.js needs to generate tangents, flip normal map y
      // https://github.com/mrdoob/three.js/issues/11438#issuecomment-507003995
      if (!geometry.attributes.tangent) material.normalScale.y *= -1;

      // Create mesh
      const mode = Primitive.mode[eid];
      let mesh: PrimitiveObject3D;
      switch (mode) {
        case WEBGL_CONSTANTS.TRIANGLES:
        case WEBGL_CONSTANTS.TRIANGLE_STRIP:
        case WEBGL_CONSTANTS.TRIANGLE_FAN:
          const primitiveMesh = Primitive.mesh[eid];
          const withMeshAndSkin = nodeWithMeshAndSkinQuery(this.#world);
          const isSkinnedMesh = withMeshAndSkin.reduce((acc, neid) => {
            const nodeMesh = NodeMesh.mesh[neid];
            return acc || nodeMesh === primitiveMesh;
          }, false);

          mesh = isSkinnedMesh ? new SkinnedMesh(geometry, material) : new Mesh(geometry, material);

          if (mesh instanceof SkinnedMesh) {
            const normalized = mesh.geometry.attributes.skinWeight.normalized;
            if (!normalized) mesh.normalizeSkinWeights();
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
          throw new Error(`Unknown primitive mode: ${mode}`);
      }

      this.#primitives.set(eid, mesh);
    });

    // Load morph targets
    this.#setMorphAttributes("position", AttributePosition);
    this.#setMorphAttributes("normal", AttributeNormal);
    this.#setMorphAttributes("tangent", AttributeTangent);

    // Set weights
    const withMesh = nodeWithMeshQuery(this.#world);
    withMesh.forEach((neid) => {
      const nodeMesh = NodeMesh.mesh[neid];
      const weights = Array.from(Node.weights[neid]);

      primitives.forEach((peid) => {
        const primitiveMesh = Primitive.mesh[peid];
        if (nodeMesh !== primitiveMesh) return;
        const primitive = this.#primitives.get(peid);
        if (!primitive) throw new Error("Primitive not found");

        primitive.updateMorphTargets();
        primitive.morphTargetInfluences?.forEach((_, i) => {
          if (!primitive.morphTargetInfluences) return;
          primitive.morphTargetInfluences[i] = weights[i] ?? 0;
        });
      });
    });
  }

  #setMorphAttributes(attributeName: string, component: ComponentType<typeof AttributeType>) {
    const query = defineQuery([MorphTarget, component]);
    const withAttribute = query(this.#world);
    withAttribute.forEach((eid) => {
      const attribute = this.#parseAttribute(AttributePosition, eid);
      const peid = MorphTarget.primitive[eid];
      const primitive = this.#primitives.get(peid);
      if (!primitive) throw new Error("Primitive not found");
      const array = primitive.geometry.morphAttributes[attributeName] ?? [];
      array.push(attribute);
      primitive.geometry.morphAttributes[attributeName] = array;
    });
  }

  #setPrimitiveAttributes(threeName: string, component: ComponentType<typeof AttributeType>) {
    const query = defineQuery([Primitive, component]);
    const withAttribute = query(this.#world);
    withAttribute.forEach((eid) => {
      const attribute = this.#parseAttribute(component, eid);
      const geometry = this.#geometies.get(eid);
      if (!geometry) throw new Error("No geometry found");
      geometry.setAttribute(threeName, attribute);
    });
  }

  #parseAttribute(component: ComponentType<typeof AttributeType>, eid: number) {
    const index = component.array[eid];
    const indices = this.#accessors[index];
    const itemSize = component.itemSize[eid];
    const normalized = Boolean(component.normalized[eid]);

    const attribute = new BufferAttribute(indices, itemSize, normalized);
    return attribute;
  }

  #parseMaterials() {
    // Create materials
    const materials = materialQuery(this.#world);
    materials.forEach((eid) => {
      this.#parseMaterial(eid);
    });

    // Color textures
    const colorTextures = materialWithColorTextureQuery(this.#world);
    colorTextures.forEach((eid) => {
      const material = this.#materials.get(eid);
      if (!material) throw new Error("Material not found");
      const texture = this.#parseTexture(ColorTexture, eid);
      texture.encoding = sRGBEncoding;
      material.map = texture;
    });

    // Emissive textures
    const emissiveTextures = materialWithEmissiveTextureQuery(this.#world);
    emissiveTextures.forEach((eid) => {
      const material = this.#materials.get(eid);
      if (!material) throw new Error("Material not found");
      const texture = this.#parseTexture(EmissiveTexture, eid);
      texture.encoding = sRGBEncoding;
      material.emissiveMap = texture;
    });

    // Metallic roughness textures
    const metallicRoughnessTextures = materialWithMetallicRoughnessTextureQuery(this.#world);
    metallicRoughnessTextures.forEach((eid) => {
      const material = this.#materials.get(eid);
      if (!material) throw new Error("Material not found");
      const texture = this.#parseTexture(MetallicRoughnessTexture, eid);
      material.metalnessMap = texture;
      material.roughnessMap = texture;
    });

    // Normal textures
    const normalTextures = materialWithNormalTextureQuery(this.#world);
    normalTextures.forEach((eid) => {
      const material = this.#materials.get(eid);
      if (!material) throw new Error("Material not found");
      const texture = this.#parseTexture(NormalTexture, eid);
      material.normalMap = texture;
      const scale = Material.normalScale[eid];
      material.normalScale.set(scale, scale);
    });

    // Occlusion textures
    const occlusionTextures = materialWithOcclusionTextureQuery(this.#world);
    occlusionTextures.forEach((eid) => {
      const material = this.#materials.get(eid);
      if (!material) throw new Error("Material not found");
      const texture = this.#parseTexture(OcclusionTexture, eid);
      material.aoMap = texture;
      material.aoMapIntensity = Material.occlusionStrength[eid];
    });
  }

  #parseMaterial(eid: number) {
    const material = new MeshStandardMaterial();
    material.needsUpdate = true;

    // Set material properties
    const baseColorFactor = [
      Material.baseColorFactor.x[eid],
      Material.baseColorFactor.y[eid],
      Material.baseColorFactor.z[eid],
      Material.baseColorFactor.w[eid],
    ];
    material.color.fromArray(baseColorFactor);

    material.opacity = Material.alpha[eid];
    material.side = Material.doubleSided[eid] ? DoubleSide : FrontSide;
    material.metalness = Material.metallicFactor[eid];
    material.roughness = Material.roughnessFactor[eid];

    const emissiveFactor = [
      Material.emissiveFactor.x[eid],
      Material.emissiveFactor.y[eid],
      Material.emissiveFactor.z[eid],
    ];
    material.emissive.fromArray(emissiveFactor);

    const alphaMode = Material.alphaMode[eid];
    if (alphaMode === AlphaMode.BLEND) {
      material.transparent = true;
      material.depthWrite = false;
    } else if (alphaMode === AlphaMode.MASK) {
      material.alphaTest = Material.alphaCutoff[eid];
    }

    this.#materials.set(eid, material);
    return material;
  }

  #parseTexture(
    component: ComponentType<{ texture: "eid"; info: typeof TextureInfo }>,
    eid: number
  ) {
    // Texture
    const teid = component.texture[eid];
    const image = Texture.image[teid];
    const bitmap = this.#images[image];

    const texture = new CanvasTexture(bitmap);
    texture.needsUpdate = true;

    // Texture info
    const magFilter = component.info.magFilter[eid];
    const minFilter = component.info.minFilter[eid];
    const wrapS = component.info.wrapS[eid];
    const wrapT = component.info.wrapT[eid];

    switch (magFilter) {
      case WEBGL_CONSTANTS.NEAREST:
        texture.magFilter = NearestFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR:
        texture.magFilter = LinearFilter;
        break;
      default:
        throw new Error(`Unknown magFilter: ${magFilter}`);
    }

    switch (minFilter) {
      case WEBGL_CONSTANTS.NEAREST:
        texture.minFilter = NearestFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR:
        texture.minFilter = LinearFilter;
        break;
      case WEBGL_CONSTANTS.NEAREST_MIPMAP_NEAREST:
        texture.minFilter = NearestMipMapNearestFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR_MIPMAP_NEAREST:
        texture.minFilter = LinearMipMapNearestFilter;
        break;
      case WEBGL_CONSTANTS.NEAREST_MIPMAP_LINEAR:
        texture.minFilter = NearestMipMapLinearFilter;
        break;
      case WEBGL_CONSTANTS.LINEAR_MIPMAP_LINEAR:
        texture.minFilter = LinearMipMapLinearFilter;
        break;
      default:
        throw new Error(`Unknown minFilter: ${minFilter}`);
    }

    switch (wrapS) {
      case WEBGL_CONSTANTS.CLAMP_TO_EDGE:
        texture.wrapS = ClampToEdgeWrapping;
        break;
      case WEBGL_CONSTANTS.MIRRORED_REPEAT:
        texture.wrapS = MirroredRepeatWrapping;
        break;
      case WEBGL_CONSTANTS.REPEAT:
        texture.wrapS = RepeatWrapping;
        break;
      default:
        throw new Error(`Unknown wrapS: ${wrapS}`);
    }

    switch (wrapT) {
      case WEBGL_CONSTANTS.CLAMP_TO_EDGE:
        texture.wrapT = ClampToEdgeWrapping;
        break;
      case WEBGL_CONSTANTS.MIRRORED_REPEAT:
        texture.wrapT = MirroredRepeatWrapping;
        break;
      case WEBGL_CONSTANTS.REPEAT:
        texture.wrapT = RepeatWrapping;
        break;
      default:
        throw new Error(`Unknown wrapT: ${wrapT}`);
    }

    return texture;
  }
}
