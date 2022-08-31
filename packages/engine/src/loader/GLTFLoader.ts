import {
  Accessor as IAccessor,
  Animation as IAnimation,
  AnimationChannel as IAnimationChannel,
  AnimationSampler as IAnimationSampler,
  Material as IMaterial,
  Mesh as IMesh,
  Node as INode,
  Primitive as IPrimitive,
  PrimitiveTarget as IPrimitiveTarget,
  Scene as IScene,
  Skin as ISkin,
  Texture as ITexture,
  Root,
  WebIO,
} from "@gltf-transform/core";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions";
import { ComponentType, addComponent, addEntity, createWorld, defineSerializer } from "bitecs";

import {
  AlphaMode,
  Animation,
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
  config,
} from "../ecs/components";
import { Assets, LoadedGLTF } from "./types";
import { loadTextureInfo } from "./utils";

const extensionDeps = {};

// Loads a glTF asset into engine ECS format
export class GLTFLoader {
  #io = new WebIO().registerExtensions(ALL_EXTENSIONS).registerDependencies(extensionDeps);
  #root: Root | null = null;

  #nodes = new Map<INode, number>();
  #meshes = new Map<IMesh, number>();
  #materials = new Map<IMaterial, number>();
  #textures = new Map<ITexture, number>();
  #accessors = new Map<IAccessor, number>();

  #imagePromises: Promise<ImageBitmap>[] = [];

  #world = createWorld(config);
  #assets: Assets = {
    names: [],
    images: [],
    accessors: [],
  };

  #serialize = defineSerializer(config);

  async load(uri: string): Promise<LoadedGLTF> {
    const document = await this.#io.read(uri);
    this.#root = document.getRoot();
    const scene = this.#root.getDefaultScene() ?? this.#root.listScenes()[0];

    this.#loadScene(scene);

    const world = this.#serialize(this.#world);

    const images = await Promise.all(this.#imagePromises);
    this.#assets.images = images;

    return { world, assets: this.#assets };
  }

  #loadScene(scene: IScene) {
    // Load nodes
    scene.listChildren().forEach((child) => {
      this.#loadNode(child);
    });

    // Load skins
    this.#loadSkins();

    // Load animations
    if (!this.#root) throw new Error("No root");
    this.#root.listAnimations().forEach((animation) => {
      this.#loadAnimation(animation);
    });
  }

  #loadAnimation(animation: IAnimation) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, Animation, eid);
    Animation.name[eid] = this.#assets.names.push(animation.getName()) - 1;

    // Load channels
    animation.listChannels().forEach((channel) => {
      this.#loadChannel(channel, eid);
    });
  }

  #loadChannel(channel: IAnimationChannel, animation: number) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, AnimationChannel, eid);
    AnimationChannel.animation[eid] = animation;

    // Set target
    const targetNode = channel.getTargetNode();
    if (!targetNode) throw new Error("No target node");
    const nodeId = this.#nodes.get(targetNode);
    if (nodeId === undefined) throw new Error("No target node entity");
    AnimationChannel.target[eid] = nodeId;

    // Set target path
    const targetPath = channel.getTargetPath();
    if (!targetPath) throw new Error("No target path");
    AnimationChannel.targetPath[eid] = TargetPath[targetPath];

    // Load sampler
    const sampler = channel.getSampler();
    if (!sampler) throw new Error("No sampler");
    AnimationChannel.sampler[eid] = this.#loadSampler(sampler);
  }

  #loadSampler(sampler: IAnimationSampler) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, AnimationSampler, eid);

    // Set interpolation
    const interpolation = sampler.getInterpolation();
    AnimationSampler.interpolation[eid] = Interpolation[interpolation];

    // Set input
    const input = sampler.getInput();
    if (!input) throw new Error("No input");
    AnimationSampler.input[eid] = this.#loadAccessor(input);

    // Set output
    const output = sampler.getOutput();
    if (!output) throw new Error("No output");
    AnimationSampler.output[eid] = this.#loadAccessor(output);

    return eid;
  }

  #loadNode(node: INode, parent?: number) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, Node, eid);
    Node.name[eid] = this.#assets.names.push(node.getName()) - 1;

    // Set parent
    if (parent !== undefined) {
      addComponent(this.#world, NodeParent, eid);
      NodeParent.parent[eid] = parent;
    }

    // Set transform
    const position = node.getTranslation();
    Node.position.x[eid] = position[0];
    Node.position.y[eid] = position[1];
    Node.position.z[eid] = position[2];

    const rotation = node.getRotation();
    Node.rotation.x[eid] = rotation[0];
    Node.rotation.y[eid] = rotation[1];
    Node.rotation.z[eid] = rotation[2];
    Node.rotation.w[eid] = rotation[3];

    const scale = node.getScale();
    Node.scale.x[eid] = scale[0];
    Node.scale.y[eid] = scale[1];
    Node.scale.z[eid] = scale[2];

    // Load mesh
    const mesh = node.getMesh();
    if (mesh) {
      addComponent(this.#world, NodeMesh, eid);
      NodeMesh.mesh[eid] = this.#loadMesh(mesh);
    }

    // Load weights
    const meshWeights = mesh?.getWeights();
    meshWeights?.forEach((value, i) => {
      Node.weights[eid][i] = value;
    });

    // Node weights overwrite mesh weights
    const nodeWeights = node.getWeights();
    nodeWeights.forEach((value, i) => {
      Node.weights[eid][i] = value;
    });

    // Load children
    node.listChildren().forEach((child) => {
      this.#loadNode(child, eid);
    });

    this.#nodes.set(node, eid);
  }

  #loadSkins() {
    this.#nodes.forEach((eid, node) => {
      const skin = node.getSkin();
      if (skin) {
        addComponent(this.#world, NodeSkin, eid);
        NodeSkin.skin[eid] = this.#loadSkin(skin);
      }
    });
  }

  #loadSkin(skin: ISkin) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, Skin, eid);

    const inverseBindMatrices = skin.getInverseBindMatrices();
    if (!inverseBindMatrices) throw new Error("No inverse bind matrices");
    Skin.inverseBindMatrices[eid] = this.#loadAccessor(inverseBindMatrices);

    // Load joints
    skin.listJoints().forEach((joint) => {
      this.#loadJoint(joint, eid);
    });

    return eid;
  }

  #loadJoint(joint: INode, skinId: number) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, SkinJoint, eid);

    // Set properties
    SkinJoint.skin[eid] = skinId;
    const jointId = this.#nodes.get(joint);
    if (jointId === undefined) throw new Error("No joint entity");
    SkinJoint.bone[eid] = jointId;
  }

  #loadMesh(mesh: IMesh) {
    // TODO: Use instanced meshes
    // const cached = this.#meshes.get(mesh);
    // if (cached !== undefined) return cached;

    // Create entity
    const eid = addEntity(this.#world);

    // Load primitives
    mesh.listPrimitives().forEach((primitive) => {
      this.#loadPrimitive(primitive, eid);
    });

    this.#meshes.set(mesh, eid);
    return eid;
  }

  #loadPrimitive(primitive: IPrimitive, mesh: number) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, Primitive, eid);

    // Set primitive properties
    Primitive.mesh[eid] = mesh;
    Primitive.mode[eid] = primitive.getMode();

    // Load material
    const material = primitive.getMaterial();
    if (material) {
      addComponent(this.#world, PrimitiveMaterial, eid);
      PrimitiveMaterial.material[eid] = this.#loadMaterial(material);
    }

    // Load indices
    const indices = primitive.getIndices();
    if (indices) {
      addComponent(this.#world, PrimitiveIndices, eid);
      PrimitiveIndices.array[eid] = this.#loadAccessor(indices);
      PrimitiveIndices.itemSize[eid] = indices.getElementSize();
      PrimitiveIndices.normalized[eid] = Number(indices.getNormalized());
    }

    // Load attributes
    primitive.listSemantics().forEach((name) => {
      const attribute = primitive.getAttribute(name);
      if (!attribute) return;

      switch (name) {
        case "POSITION":
          this.#loadAttribute(attribute, AttributePosition, eid);
          break;
        case "NORMAL":
          this.#loadAttribute(attribute, AttributeNormal, eid);
          break;
        case "TANGENT":
          this.#loadAttribute(attribute, AttributeTangent, eid);
          break;
        case "TEXCOORD_0":
          this.#loadAttribute(attribute, AttributeUV, eid);
          break;
        case "TEXCOORD_1":
          this.#loadAttribute(attribute, AttributeUV2, eid);
          break;
        case "COLOR_0":
          this.#loadAttribute(attribute, AttributeColor, eid);
          break;
        case "JOINTS_0":
          this.#loadAttribute(attribute, AttributeSkinIndex, eid);
          break;
        case "WEIGHTS_0":
          this.#loadAttribute(attribute, AttributeSkinWeight, eid);
          break;
        default:
          throw new Error(`Unsupported primitive attribute: ${name}`);
      }
    });

    // Load morph targets
    primitive.listTargets().forEach((target) => {
      this.#loadMorphTarget(target, eid);
    });

    return eid;
  }

  #loadMorphTarget(target: IPrimitiveTarget, primitive: number) {
    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, MorphTarget, eid);
    MorphTarget.primitive[eid] = primitive;

    // Load attributes
    target.listSemantics().forEach((name) => {
      const attribute = target.getAttribute(name);
      if (!attribute) return;

      switch (name) {
        case "POSITION":
          this.#loadAttribute(attribute, AttributePosition, eid);
          break;
        case "NORMAL":
          this.#loadAttribute(attribute, AttributeNormal, eid);
          break;
        case "TANGENT":
          this.#loadAttribute(attribute, AttributeTangent, eid);
          break;
        default:
          throw new Error(`Unsupported primitive attribute: ${name}`);
      }
    });
  }

  #loadAttribute(
    attribute: IAccessor,
    component: ComponentType<typeof AttributeType>,
    eid: number
  ) {
    addComponent(this.#world, component, eid);
    component.array[eid] = this.#loadAccessor(attribute);
    component.itemSize[eid] = attribute.getElementSize();
    component.normalized[eid] = Number(attribute.getNormalized());
  }

  #loadMaterial(material: IMaterial) {
    const cached = this.#materials.get(material);
    if (cached !== undefined) return cached;

    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, Material, eid);
    Material.name[eid] = this.#assets.names.push(material.getName()) - 1;

    // Set material properties
    Material.doubleSided[eid] = Number(material.getDoubleSided());

    // Alpha
    Material.alpha[eid] = material.getAlpha();
    Material.alphaCutoff[eid] = material.getAlphaCutoff();
    Material.alphaMode[eid] = AlphaMode[material.getAlphaMode()];

    // Base color
    const colorFactor = material.getBaseColorFactor();
    Material.baseColorFactor.x[eid] = colorFactor[0];
    Material.baseColorFactor.y[eid] = colorFactor[1];
    Material.baseColorFactor.z[eid] = colorFactor[2];
    Material.baseColorFactor.w[eid] = colorFactor[3];
    Material.baseColorHex[eid] = material.getBaseColorHex();
    // Color texture
    const colorTexture = material.getBaseColorTexture();
    const colorInfo = material.getBaseColorTextureInfo();
    if (colorTexture && colorInfo) {
      addComponent(this.#world, ColorTexture, eid);
      ColorTexture.texture[eid] = this.#loadTexture(colorTexture);
      loadTextureInfo(colorInfo, ColorTexture.info, eid);
    }

    // Emissive
    const emissiveFactor = material.getEmissiveFactor();
    Material.emissiveFactor.x[eid] = emissiveFactor[0];
    Material.emissiveFactor.y[eid] = emissiveFactor[1];
    Material.emissiveFactor.z[eid] = emissiveFactor[2];
    Material.emissiveHex[eid] = material.getEmissiveHex();
    // Emissive texture
    const emissiveTexture = material.getEmissiveTexture();
    const emissiveInfo = material.getEmissiveTextureInfo();
    if (emissiveTexture && emissiveInfo) {
      addComponent(this.#world, EmissiveTexture, eid);
      EmissiveTexture.texture[eid] = this.#loadTexture(emissiveTexture);
      loadTextureInfo(emissiveInfo, EmissiveTexture.info, eid);
    }

    // Metallic roughness
    Material.metallicFactor[eid] = material.getMetallicFactor();
    Material.roughnessFactor[eid] = material.getRoughnessFactor();

    // Metallic roughness texture
    const metallicRoughnessTexture = material.getMetallicRoughnessTexture();
    const metallicRoughnessInfo = material.getMetallicRoughnessTextureInfo();
    if (metallicRoughnessTexture && metallicRoughnessInfo) {
      addComponent(this.#world, MetallicRoughnessTexture, eid);
      MetallicRoughnessTexture.texture[eid] = this.#loadTexture(metallicRoughnessTexture);
      loadTextureInfo(metallicRoughnessInfo, MetallicRoughnessTexture.info, eid);
    }

    // Normal
    Material.normalScale[eid] = material.getNormalScale();
    // Normal texture
    const normalTexture = material.getNormalTexture();
    const normalInfo = material.getNormalTextureInfo();
    if (normalTexture && normalInfo) {
      addComponent(this.#world, NormalTexture, eid);
      NormalTexture.texture[eid] = this.#loadTexture(normalTexture);
      loadTextureInfo(normalInfo, NormalTexture.info, eid);
    }

    // Occlusion
    Material.occlusionStrength[eid] = material.getOcclusionStrength();
    // Occlusion texture
    const occlusionTexture = material.getOcclusionTexture();
    const occlusionInfo = material.getOcclusionTextureInfo();
    if (occlusionTexture && occlusionInfo) {
      addComponent(this.#world, OcclusionTexture, eid);
      OcclusionTexture.texture[eid] = this.#loadTexture(occlusionTexture);
      loadTextureInfo(occlusionInfo, OcclusionTexture.info, eid);
    }

    this.#materials.set(material, eid);
    return eid;
  }

  #loadTexture(texture: ITexture) {
    const cached = this.#textures.get(texture);
    if (cached !== undefined) return cached;

    // Create entity
    const eid = addEntity(this.#world);
    addComponent(this.#world, Texture, eid);
    Texture.name[eid] = this.#assets.names.push(texture.getName()) - 1;

    // Set image
    const mimeType = texture.getMimeType();
    const image = texture.getImage();
    if (image) {
      const blob = new Blob([image], { type: mimeType });
      const bitmap = createImageBitmap(blob);
      Texture.image[eid] = this.#imagePromises.push(bitmap) - 1;
    } else throw new Error("Texture has no image");

    this.#textures.set(texture, eid);
    return eid;
  }

  #loadAccessor(accessor: IAccessor) {
    const cached = this.#accessors.get(accessor);
    if (cached !== undefined) return cached;

    const array = accessor.getArray();
    if (!array) throw new Error("Accessor array is null");

    const index = this.#assets.accessors.push(array) - 1;
    this.#accessors.set(accessor, index);
    return index;
  }
}
