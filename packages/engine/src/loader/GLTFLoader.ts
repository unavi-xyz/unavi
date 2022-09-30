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
  Root,
  Scene as IScene,
  Texture as ITexture,
  TextureInfo as ITextureInfo,
  WebIO,
} from "@gltf-transform/core";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions";
import { nanoid } from "nanoid";

import {
  AnimationChannel,
  AnimationSampler,
  Entity,
  Material,
  PrimitiveMesh,
  Scene,
  Texture,
} from "../scene";
import { Accessor } from "../scene/Accessor";
import { Animation } from "../scene/Animation";
import { Image } from "../scene/Image";

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

  #map = {
    nodes: new Map<INode, string>(),
    meshes: new Map<IMesh, Entity[]>(),
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
    if (!scene) throw new Error("No scene");

    this.#loadScene(scene);

    await Promise.all(this.#pending);

    return this.#scene;
  }

  #loadScene(scene: IScene) {
    // Load nodes
    scene.listChildren().forEach((child) => {
      this.#loadNode(child);
    });

    // Load skin data
    this.#map.nodes.forEach((_, node) => {
      const skin = node.getSkin();

      if (skin) {
        const mesh = node.getMesh();
        if (!mesh) throw new Error("No mesh");

        const meshEntities = this.#map.meshes.get(mesh);
        if (!meshEntities) throw new Error("No mesh entities");

        meshEntities.forEach((meshEntity) => {
          if (meshEntity.mesh?.type !== "Primitive")
            throw new Error("Wrong mesh");

          // Load inverse bind matrices
          const inverseBindMatrices = skin.getInverseBindMatrices();
          if (!inverseBindMatrices) throw new Error("No inverse bind matrices");
          const inverseBindMatricesId = this.#loadAccessor(inverseBindMatrices);

          // Load joints
          const jointIds = skin.listJoints().map((joint) => {
            const entityId = this.#map.nodes.get(joint);
            if (entityId === undefined) throw new Error("No joint entity");
            return entityId;
          });

          // Set skin data
          meshEntity.mesh.skin = {
            inverseBindMatricesId,
            jointIds,
          };
        });
      }
    });

    // Load animations
    if (!this.#root) throw new Error("No root");
    this.#root.listAnimations().forEach((animation) => {
      this.#loadAnimation(animation);
    });
  }

  #loadAnimation(animation: IAnimation) {
    // Create entity
    const animationState = new Animation();
    animationState.isInternal = true;
    animationState.name = animation.getName();

    // Load channels
    animationState.channels = animation
      .listChannels()
      .map((channel) => this.#loadChannel(channel));

    // Add to scene
    this.#scene.addAnimation(animationState);
  }

  #loadChannel(channel: IAnimationChannel): AnimationChannel {
    // Get target
    const targetNode = channel.getTargetNode();
    if (!targetNode) throw new Error("No target node");

    const targetId = this.#map.nodes.get(targetNode);
    if (targetId === undefined) throw new Error("No target node entity");

    // Load sampler
    const sampler = channel.getSampler();
    if (!sampler) throw new Error("No sampler");

    const channelState: AnimationChannel = {
      targetId,
      path: channel.getTargetPath(),
      sampler: this.#loadSampler(sampler),
    };
    return channelState;
  }

  #loadSampler(sampler: IAnimationSampler): AnimationSampler {
    // Get interpolation
    const interpolation = sampler.getInterpolation();

    // Get input
    const input = sampler.getInput();
    if (!input) throw new Error("No input");
    const inputId = this.#loadAccessor(input);

    // Get output
    const output = sampler.getOutput();
    if (!output) throw new Error("No output");
    const outputId = this.#loadAccessor(output);

    const samplerState: AnimationSampler = {
      interpolation,
      inputId,
      outputId,
    };

    return samplerState;
  }

  #loadNode(node: INode, parentId?: string): Entity {
    // Create entity
    const entity = new Entity();
    entity.isInternal = true;
    entity.name = node.getName();

    // Set parent
    if (parentId !== undefined) entity.parentId = parentId;

    // Set transform
    entity.position = node.getTranslation();
    entity.rotation = node.getRotation();
    entity.scale = node.getScale();

    // Load mesh
    const mesh = node.getMesh();
    if (mesh) {
      const meshEntities = this.#loadMesh(mesh);

      const nodeWeights = node.getWeights();
      const weights = nodeWeights.length > 0 ? nodeWeights : mesh.getWeights();

      meshEntities.forEach((meshEntity) => {
        if (meshEntity.mesh?.type !== "Primitive") throw new Error("No mesh");

        // Set weights
        meshEntity.mesh.weights = weights;

        // Add to node
        meshEntity.parentId = entity.id;
      });
    }

    // Load children
    node.listChildren().forEach((child) => {
      this.#loadNode(child, entity.id);
    });

    // Add to scene
    this.#scene.addEntity(entity);

    this.#map.nodes.set(node, entity.id);
    return entity;
  }

  #loadMesh(mesh: IMesh): Entity[] {
    // TODO: Support instanced meshes
    // const cached = this.#map.meshes.get(mesh);
    // if (cached !== undefined) return cached;

    // Load primitives
    const primitives = mesh
      .listPrimitives()
      .map((primitive) => this.#loadPrimitive(primitive));

    this.#map.meshes.set(mesh, primitives);
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
    primitive.listTargets().forEach((target) => {
      this.#loadMorphTarget(target, mesh);
    });

    // Add to scene
    this.#scene.addEntity(entity);

    return entity;
  }

  #loadMorphTarget(target: IPrimitiveTarget, mesh: PrimitiveMesh) {
    // Load attributes
    target.listSemantics().forEach((name) => {
      const attribute = target.getAttribute(name);
      if (!attribute) return;

      const accessorId = this.#loadAccessor(attribute);

      switch (name) {
        case "POSITION":
          mesh.morphPositionIds = [...mesh.morphPositionIds, accessorId];
          break;
        case "NORMAL":
          mesh.morphNormalIds = [...mesh.morphNormalIds, accessorId];
          break;
        case "TANGENT":
          mesh.morphTangentIds = [...mesh.morphTangentIds, accessorId];
          break;
        default:
          throw new Error(`Unsupported primitive attribute: ${name}`);
      }
    });
  }

  #loadMaterial(material: IMaterial): Material {
    const cached = this.#map.materials.get(material);
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
      materialState.colorTexture = this.#loadTexture(colorTexture, colorInfo);

    // Emissive
    materialState.emissive = material.getEmissiveFactor();

    // Emissive texture
    const emissiveTexture = material.getEmissiveTexture();
    const emissiveInfo = material.getEmissiveTextureInfo();
    if (emissiveTexture && emissiveInfo)
      materialState.emissiveTexture = this.#loadTexture(
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
      materialState.metallicRoughnessTexture = this.#loadTexture(
        metallicRoughnessTexture,
        metallicRoughnessInfo
      );

    // Normal scale
    materialState.normalScale = material.getNormalScale();

    // Normal texture
    const normalTexture = material.getNormalTexture();
    const normalInfo = material.getNormalTextureInfo();
    if (normalTexture && normalInfo)
      materialState.normalTexture = this.#loadTexture(
        normalTexture,
        normalInfo
      );

    // Occlusion strength
    materialState.occlusionStrength = material.getOcclusionStrength();

    // Occlusion texture
    const occlusionTexture = material.getOcclusionTexture();
    const occlusionInfo = material.getOcclusionTextureInfo();
    if (occlusionTexture && occlusionInfo)
      materialState.occlusionTexture = this.#loadTexture(
        occlusionTexture,
        occlusionInfo
      );

    // Add to scene
    this.#scene.addMaterial(materialState);

    this.#map.materials.set(material, materialState);
    return materialState;
  }

  #loadTexture(texture: ITexture, info: ITextureInfo): Texture {
    // Create texture
    const textureState = new Texture();
    textureState.imageId = this.#loadTextureImage(texture);

    const magFilter = info.getMagFilter();
    if (magFilter !== null) textureState.magFilter = magFilter;

    const minFilter = info.getMinFilter();
    if (minFilter !== null) textureState.minFilter = minFilter;

    const wrapS = info.getWrapS();
    textureState.wrapS = wrapS;

    const wrapT = info.getWrapT();
    textureState.wrapT = wrapT;

    return textureState;
  }

  #loadTextureImage(texture: ITexture): string {
    const cached = this.#map.images.get(texture);
    if (cached !== undefined) return cached;

    const id = nanoid();

    // Load image
    const mimeType = texture.getMimeType();
    const imageArray = texture.getImage();

    if (imageArray) {
      this.#map.images.set(texture, id);

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
    const cached = this.#map.accessors.get(accessor);
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

    this.#map.accessors.set(accessor, accessorState.id);
    return accessorState.id;
  }
}
