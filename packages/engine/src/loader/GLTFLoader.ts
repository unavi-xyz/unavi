import {
  Accessor as IAccessor,
  Material as IMaterial,
  Mesh as IMesh,
  Node as INode,
  Primitive as IPrimitive,
  PrimitiveTarget as IPrimitiveTarget,
  Root,
  Scene as IScene,
  Skin as ISkin,
  Texture as ITexture,
  TextureInfo as ITextureInfo,
  WebIO,
} from "@gltf-transform/core";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions";
import { nanoid } from "nanoid";

import { Entity, Material, PrimitiveMesh, Scene, Texture } from "../scene";
import { Accessor } from "../scene/Accessor";
import { Image } from "../scene/Image";
import { quaternionToEuler } from "./quaternionToEuler";

const extensionDeps = {};

/*
 * Loads a GLTF model into the engine's internal scene format.
 */
export class GLTFLoader {
  #scene = new Scene();

  #io = new WebIO()
    .registerExtensions(ALL_EXTENSIONS)
    .registerDependencies(extensionDeps);
  #root: Root | null = null;

  #cache = {
    materials: new Map<IMaterial, Material>(),
    accessors: new Map<IAccessor, string>(),
    images: new Map<ITexture, string>(),
  };

  #pending: Promise<void>[] = [];

  async load(uri: string): Promise<Scene> {
    const document = await this.#io.read(uri);
    this.#root = document.getRoot();

    // Load only one scene
    const scene = this.#root.getDefaultScene() ?? this.#root.listScenes()[0];

    this.#loadScene(scene);

    await Promise.all(this.#pending);

    return this.#scene;
  }

  #loadScene(scene: IScene) {
    // Load nodes
    scene.listChildren().forEach((child) => {
      this.#loadNode(child);
    });

    // Load skins
    // this.#loadSkins();

    // Load animations
    // if (!this.#root) throw new Error("No root");
    // this.#root.listAnimations().forEach((animation) => {
    //   this.#loadAnimation(animation);
    // });
  }

  // #loadAnimation(animation: IAnimation) {
  //   // Create entity
  //   const eid = addEntity(this.#world);
  //   addComponent(this.#world, Animation, eid);
  //   Animation.name[eid] = this.#assets.names.push(animation.getName()) - 1;

  //   // Load channels
  //   animation.listChannels().forEach((channel) => {
  //     this.#loadChannel(channel, eid);
  //   });
  // }

  // #loadChannel(channel: IAnimationChannel, animation: number) {
  //   // Create entity
  //   const eid = addEntity(this.#world);
  //   addComponent(this.#world, AnimationChannel, eid);
  //   AnimationChannel.animation[eid] = animation;

  //   // Set target
  //   const targetNode = channel.getTargetNode();
  //   if (!targetNode) throw new Error("No target node");
  //   const nodeId = this.#nodes.get(targetNode);
  //   if (nodeId === undefined) throw new Error("No target node entity");
  //   AnimationChannel.target[eid] = nodeId;

  //   // Set target path
  //   const targetPath = channel.getTargetPath();
  //   if (!targetPath) throw new Error("No target path");
  //   AnimationChannel.targetPath[eid] = TargetPath[targetPath];

  //   // Load sampler
  //   const sampler = channel.getSampler();
  //   if (!sampler) throw new Error("No sampler");
  //   AnimationChannel.sampler[eid] = this.#loadSampler(sampler);
  // }

  // #loadSampler(sampler: IAnimationSampler) {
  //   // Create entity
  //   const eid = addEntity(this.#world);
  //   addComponent(this.#world, AnimationSampler, eid);

  //   // Set interpolation
  //   const interpolation = sampler.getInterpolation();
  //   AnimationSampler.interpolation[eid] = Interpolation[interpolation];

  //   // Set input
  //   const input = sampler.getInput();
  //   if (!input) throw new Error("No input");
  //   AnimationSampler.input[eid] = this.#loadAccessor(input);

  //   // Set output
  //   const output = sampler.getOutput();
  //   if (!output) throw new Error("No output");
  //   AnimationSampler.output[eid] = this.#loadAccessor(output);

  //   return eid;
  // }

  #loadNode(node: INode, parentId?: string): Entity {
    // Create entity
    const entity = new Entity();
    entity.isInternal = true;
    entity.name = node.getName();

    // Set parent
    if (parentId !== undefined) entity.parentId = parentId;

    // Set transform
    entity.position = node.getTranslation();
    entity.rotation = quaternionToEuler(node.getRotation());
    entity.scale = node.getScale();

    // Load mesh
    const mesh = node.getMesh();
    if (mesh) {
      const meshEntities = this.#loadMesh(mesh);
      // Add mesh entities as children
      meshEntities.forEach((meshEntity) => (meshEntity.parentId = entity.id));
    }

    // Load weights
    // const meshWeights = mesh?.getWeights();
    // meshWeights?.forEach((value, i) => {
    //   Node.weights[eid][i] = value;
    // });

    // Node weights overwrite mesh weights
    // const nodeWeights = node.getWeights();
    // nodeWeights.forEach((value, i) => {
    //   Node.weights[eid][i] = value;
    // });

    // Load children
    node.listChildren().forEach((child) => {
      this.#loadNode(child, entity.id);
    });

    // Add to scene
    this.#scene.addEntity(entity);

    return entity;
  }

  #loadSkins() {
    // this.#nodes.forEach((eid, node) => {
    //   const skin = node.getSkin();
    //   if (skin) {
    //     addComponent(this.#world, NodeSkin, eid);
    //     NodeSkin.skin[eid] = this.#loadSkin(skin);
    //   }
    // });
  }

  #loadSkin(skin: ISkin) {
    // // Create entity
    // const eid = addEntity(this.#world);
    // addComponent(this.#world, Skin, eid);
    // const inverseBindMatrices = skin.getInverseBindMatrices();
    // if (!inverseBindMatrices) throw new Error("No inverse bind matrices");
    // Skin.inverseBindMatrices[eid] = this.#loadAccessor(inverseBindMatrices);
    // // Load joints
    // skin.listJoints().forEach((joint) => {
    //   this.#loadJoint(joint, eid);
    // });
    // return eid;
  }

  #loadJoint(joint: INode, skinId: number) {
    // // Create entity
    // const eid = addEntity(this.#world);
    // addComponent(this.#world, SkinJoint, eid);
    // // Set properties
    // SkinJoint.skin[eid] = skinId;
    // const jointId = this.#nodes.get(joint);
    // if (jointId === undefined) throw new Error("No joint entity");
    // SkinJoint.bone[eid] = jointId;
  }

  #loadMesh(mesh: IMesh): Entity[] {
    // TODO: Use instanced meshes?
    // const cached = this.#meshes.get(mesh);
    // if (cached !== undefined) return cached;

    // Load primitives
    const primitives = mesh
      .listPrimitives()
      .map((primitive) => this.#loadPrimitive(primitive));

    return primitives;
  }

  #loadPrimitive(primitive: IPrimitive): Entity {
    // Create entity
    const entity = new Entity();
    entity.isInternal = true;
    entity.name = primitive.getName();

    // Create mesh
    const mesh = new PrimitiveMesh();
    entity.mesh = mesh;
    mesh.mode = primitive.getMode();

    // Load material
    const material = primitive.getMaterial();
    if (material) entity.materialId = this.#loadMaterial(material).id;

    // Load indices
    const indices = primitive.getIndices();
    if (indices) mesh.indicesId = this.#loadAccessor(indices);

    // Load attributes
    primitive.listSemantics().forEach((name) => {
      const attribute = primitive.getAttribute(name);
      if (!attribute) return;

      switch (name) {
        case "POSITION":
          mesh.POSITION = this.#loadAccessor(attribute);
          break;
        case "NORMAL":
          mesh.NORMAL = this.#loadAccessor(attribute);
          break;
        case "TANGENT":
          mesh.TANGENT = this.#loadAccessor(attribute);
          break;
        case "TEXCOORD_0":
          mesh.TEXCOORD_0 = this.#loadAccessor(attribute);
          break;
        case "TEXCOORD_1":
          mesh.TEXCOORD_1 = this.#loadAccessor(attribute);
          break;
        case "COLOR_0":
          mesh.COLOR_0 = this.#loadAccessor(attribute);
          break;
        case "JOINTS_0":
          mesh.JOINTS_0 = this.#loadAccessor(attribute);
          break;
        case "WEIGHTS_0":
          mesh.WEIGHTS_0 = this.#loadAccessor(attribute);
          break;
        default:
          throw new Error(`Unsupported primitive attribute: ${name}`);
      }
    });

    // Load morph targets
    // primitive.listTargets().forEach((target) => {
    //   this.#loadMorphTarget(target, eid);
    // });

    // Add to scene
    this.#scene.addEntity(entity);

    return entity;
  }

  #loadMorphTarget(target: IPrimitiveTarget, primitive: number) {
    // // Create entity
    // const eid = addEntity(this.#world);
    // addComponent(this.#world, MorphTarget, eid);
    // MorphTarget.primitive[eid] = primitive;
    // // Load attributes
    // target.listSemantics().forEach((name) => {
    //   const attribute = target.getAttribute(name);
    //   if (!attribute) return;
    //   switch (name) {
    //     case "POSITION":
    //       this.#loadAttribute(attribute, AttributePosition, eid);
    //       break;
    //     case "NORMAL":
    //       this.#loadAttribute(attribute, AttributeNormal, eid);
    //       break;
    //     case "TANGENT":
    //       this.#loadAttribute(attribute, AttributeTangent, eid);
    //       break;
    //     default:
    //       throw new Error(`Unsupported primitive attribute: ${name}`);
    //   }
    // });
  }

  #loadMaterial(material: IMaterial): Material {
    const cached = this.#cache.materials.get(material);
    if (cached !== undefined) return cached;

    // Create entity
    const materialState = new Material();
    materialState.isInternal = true;
    materialState.name = material.getName();
    materialState.doubleSided = material.getDoubleSided();
    materialState.color = material.getBaseColorFactor();

    // Alpha
    materialState.alpha = material.getAlpha();
    materialState.alphaCutoff = material.getAlphaCutoff();
    materialState.alphaMode = material.getAlphaMode();

    // Color texture
    const colorTexture = material.getBaseColorTexture();
    const colorInfo = material.getBaseColorTextureInfo();
    if (colorTexture && colorInfo)
      materialState.colorTextureId = this.#loadTexture(colorTexture, colorInfo);

    // Emissive
    materialState.emissive = material.getEmissiveFactor();

    // Emissive texture
    const emissiveTexture = material.getEmissiveTexture();
    const emissiveInfo = material.getEmissiveTextureInfo();
    if (emissiveTexture && emissiveInfo)
      materialState.emissiveTextureId = this.#loadTexture(
        emissiveTexture,
        emissiveInfo
      );

    // Metallic roughness
    materialState.metalness = material.getMetallicFactor();
    materialState.roughness = material.getRoughnessFactor();

    // Metallic roughness texture
    const metallicRoughnessTexture = material.getMetallicRoughnessTexture();
    const metallicRoughnessInfo = material.getMetallicRoughnessTextureInfo();
    if (metallicRoughnessTexture && metallicRoughnessInfo)
      materialState.metallicRoughnessTextureId = this.#loadTexture(
        metallicRoughnessTexture,
        metallicRoughnessInfo
      );

    // Normal scale
    materialState.normalScale = material.getNormalScale();

    // Normal texture
    const normalTexture = material.getNormalTexture();
    const normalInfo = material.getNormalTextureInfo();
    if (normalTexture && normalInfo)
      materialState.normalTextureId = this.#loadTexture(
        normalTexture,
        normalInfo
      );

    // Occlusion strength
    materialState.occlusionStrength = material.getOcclusionStrength();

    // Occlusion texture
    const occlusionTexture = material.getOcclusionTexture();
    const occlusionInfo = material.getOcclusionTextureInfo();
    if (occlusionTexture && occlusionInfo)
      materialState.occlusionTextureId = this.#loadTexture(
        occlusionTexture,
        occlusionInfo
      );

    // Add to scene
    this.#scene.addMaterial(materialState);

    this.#cache.materials.set(material, materialState);
    return materialState;
  }

  #loadTexture(texture: ITexture, info: ITextureInfo): string {
    // Create texture
    const textureState = new Texture();
    textureState.isInternal = true;
    textureState.name = texture.getName();

    textureState.sourceId = this.#loadTextureImage(texture);

    const magFilter = info.getMagFilter();
    if (magFilter !== null) textureState.magFilter = magFilter;

    const minFilter = info.getMinFilter();
    if (minFilter !== null) textureState.minFilter = minFilter;

    const wrapS = info.getWrapS();
    textureState.wrapS = wrapS;

    const wrapT = info.getWrapT();
    textureState.wrapT = wrapT;

    // Add to scene
    this.#scene.addTexture(textureState);

    return textureState.id;
  }

  #loadTextureImage(texture: ITexture): string {
    const cached = this.#cache.images.get(texture);
    if (cached !== undefined) return cached;

    const id = nanoid();

    // Load image
    const mimeType = texture.getMimeType();
    const imageArray = texture.getImage();

    if (imageArray) {
      this.#cache.images.set(texture, id);

      // Create image bitmap asynchronously
      const blob = new Blob([imageArray], { type: mimeType });
      const promise = createImageBitmap(blob).then((bitmap) => {
        const image = new Image({ id, bitmap });
        image.isInternal = true;
        this.#scene.addImage(image);
      });

      this.#pending.push(promise);

      return id;
    } else {
      throw new Error("Texture has no image");
    }
  }

  #loadAccessor(accessor: IAccessor): string {
    const cached = this.#cache.accessors.get(accessor);
    if (cached !== undefined) return cached;

    // Create accessor
    const array = accessor.getArray();
    if (!array) throw new Error("Accessor array is null");

    const elementSize = accessor.getElementSize();
    const normalized = accessor.getNormalized();

    const accessorState = new Accessor({
      array,
      elementSize,
      normalized,
    });
    accessorState.isInternal = true;

    // Add to scene
    this.#scene.addAccessor(accessorState);

    this.#cache.accessors.set(accessor, accessorState.id);
    return accessorState.id;
  }
}
