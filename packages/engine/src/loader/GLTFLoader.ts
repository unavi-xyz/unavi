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
import { nanoid } from "nanoid";

import { extensions } from "../gltf/constants";
import { Collider } from "../gltf/extensions/Collider/Collider";
import { ColliderExtension } from "../gltf/extensions/Collider/ColliderExtension";
import { SpawnPointExtension } from "../gltf/extensions/SpawnPoint/SpawnPointExtension";
import {
  AnimationChannel,
  AnimationSampler,
  BoxCollider,
  CylinderCollider,
  HullCollider,
  Material,
  MeshCollider,
  Node,
  Primitive,
  Scene,
  SphereCollider,
  Texture,
} from "../scene";
import { Accessor } from "../scene/Accessor";
import { Animation } from "../scene/Animation";
import { Image } from "../scene/Image";
import { PrimitivesMesh } from "../scene/mesh/PrimitivesMesh";

/*
 * Loads a GLTF model into the engine's internal scene format.
 */
export class GLTFLoader {
  #scene = new Scene();
  #io: WebIO | null = null;
  #root: Root | null = null;

  #map = {
    nodes: new Map<INode, string>(),
    meshes: new Map<IMesh, PrimitivesMesh>(),
    materials: new Map<IMaterial, Material>(),
    accessors: new Map<IAccessor, string>(),
    images: new Map<ITexture, string>(),
  };

  #pending: Promise<void>[] = [];

  #spawnId: string | null = null;

  async load(uri: string): Promise<Scene> {
    this.#io = new WebIO().registerExtensions(extensions).registerDependencies({
      // @ts-ignore
      "draco3d.decoder": await new DracoDecoderModule(),
    });

    const res = await fetch(uri);
    const mimeType = res.headers.get("Content-Type");

    const readJSON = async () => {
      if (!this.#io) throw new Error("No io");
      const document = await this.#io.read(uri);
      this.#root = document.getRoot();
    };

    const readBinary = async () => {
      if (!this.#io) throw new Error("No io");
      const buffer = await res.arrayBuffer();
      const array = new Uint8Array(buffer);
      const document = await this.#io.readBinary(array);
      this.#root = document.getRoot();
    };

    switch (mimeType) {
      case "model/gltf+json": {
        await readJSON();
        break;
      }

      case "model/gltf-binary": {
        await readBinary();
        break;
      }

      default: {
        // If no mime type is provided, try to read as json first, then binary if it fails
        try {
          await readJSON();
        } catch (e) {
          await readBinary();
        }
      }
    }

    if (!this.#root) throw new Error("Failed to load file");

    const scene = this.#root.getDefaultScene() ?? this.#root.listScenes()[0];
    if (!scene) throw new Error("No scene");

    this.#loadScene(scene);

    await Promise.all(this.#pending);

    this.#scene.spawnId = this.#spawnId;

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
        const gltfMesh = node.getMesh();
        if (!gltfMesh) throw new Error("No mesh");

        const mesh = this.#map.meshes.get(gltfMesh);
        if (!mesh) throw new Error("No mesh");

        mesh.primitives.forEach((primitive) => {
          // Load inverse bind matrices
          const inverseBindMatrices = skin.getInverseBindMatrices();
          if (!inverseBindMatrices) throw new Error("No inverse bind matrices");
          const inverseBindMatricesId = this.#loadAccessor(inverseBindMatrices);

          // Load joints
          const jointIds = skin.listJoints().map((joint) => {
            const nodeId = this.#map.nodes.get(joint);
            if (nodeId === undefined) throw new Error("No joint node");
            return nodeId;
          });

          // Set skin data
          primitive.skin = {
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
    // Create node
    const animationState = new Animation();
    animationState.isInternal = true;
    animationState.name = animation.getName();

    // Load channels
    animationState.channels = animation.listChannels().map((channel) => this.#loadChannel(channel));

    // Add to scene
    this.#scene.addAnimation(animationState);
  }

  #loadChannel(channel: IAnimationChannel): AnimationChannel {
    // Get target
    const targetNode = channel.getTargetNode();
    if (!targetNode) throw new Error("No target node");

    const targetId = this.#map.nodes.get(targetNode);
    if (targetId === undefined) throw new Error("No target node node");

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

  #loadNode(gltfNode: INode, parentId?: string): Node {
    // Create node
    const node = new Node();
    node.isInternal = true;
    node.name = gltfNode.getName();

    // Set parent
    if (parentId !== undefined) node.parentId = parentId;

    // Set transform
    node.position = gltfNode.getTranslation();
    node.rotation = gltfNode.getRotation();
    node.scale = gltfNode.getScale();

    // Load mesh
    const gltfMesh = gltfNode.getMesh();
    if (gltfMesh) {
      const mesh = this.#loadMesh(gltfMesh);

      // Set mesh
      node.meshId = mesh.id;

      // If the node has weights, apply them to the mesh
      const nodeWeights = gltfNode.getWeights();
      if (nodeWeights.length > 0) {
        // If the mesh is used by multiple nodes, clone the mesh
        const isSingleUse = gltfMesh.listParents().length === 1;

        if (isSingleUse) {
          mesh.primitives.forEach((primitive) => (primitive.weights = nodeWeights));
        } else {
          const gltfMeshCopy = gltfMesh.clone();
          const meshCopy = this.#loadMesh(gltfMeshCopy);
          meshCopy.primitives.forEach((primitive) => (primitive.weights = nodeWeights));

          // Set new mesh
          node.meshId = meshCopy.id;
        }
      }
    }

    // Load collider
    const collider = gltfNode.getExtension(ColliderExtension.EXTENSION_NAME);

    if (collider instanceof Collider) {
      switch (collider.getType()) {
        case "box": {
          const box = new BoxCollider();
          box.size = collider.getSize() ?? [1, 1, 1];
          node.collider = box;
          break;
        }

        case "sphere": {
          const sphere = new SphereCollider();
          sphere.radius = collider.getRadius() ?? 0.5;
          node.collider = sphere;
          break;
        }

        case "cylinder": {
          const cylinder = new CylinderCollider();
          cylinder.radius = collider.getRadius() ?? 0.5;
          cylinder.height = collider.getHeight() ?? 1;
          node.collider = cylinder;
          break;
        }

        case "hull": {
          if (!node.meshId) {
            console.warn("No mesh for hull collider");
            break;
          }

          const hull = new HullCollider();
          hull.meshId = node.meshId;
          node.collider = hull;
          break;
        }

        case "mesh": {
          if (!node.meshId) {
            console.warn("No mesh for mesh collider");
            break;
          }

          const mesh = new MeshCollider();
          mesh.meshId = node.meshId;
          node.collider = mesh;
          break;
        }

        default: {
          console.warn(`Collider type ${collider.getType()} not supported`);
        }
      }
    }

    // Load spawn
    const spawn = gltfNode.getExtension(SpawnPointExtension.EXTENSION_NAME);
    if (spawn) this.#spawnId = node.id;

    // Load children
    gltfNode.listChildren().forEach((child) => this.#loadNode(child, node.id));

    // Add to scene
    this.#scene.addNode(node);

    this.#map.nodes.set(gltfNode, node.id);
    return node;
  }

  #loadMesh(gltfMesh: IMesh): PrimitivesMesh {
    const mesh = new PrimitivesMesh();
    mesh.isInternal = true;
    mesh.name = gltfMesh.getName();

    // Load primitives
    mesh.primitives = gltfMesh.listPrimitives().map((primitive) => this.#loadPrimitive(primitive));

    // Set weights
    const weights = gltfMesh.getWeights();
    mesh.primitives.forEach((primitive) => (primitive.weights = weights));

    // Add to scene
    this.#scene.addMesh(mesh);

    this.#map.meshes.set(gltfMesh, mesh);
    return mesh;
  }

  #loadPrimitive(gltfPrimitive: IPrimitive): Primitive {
    // Create primitive
    const primitive = new Primitive();
    primitive.isInternal = true;
    primitive.name = gltfPrimitive.getName();

    primitive.mode = gltfPrimitive.getMode();

    // Load material
    const material = gltfPrimitive.getMaterial();
    if (material) primitive.materialId = this.#loadMaterial(material).id;

    // Load indices
    const indices = gltfPrimitive.getIndices();
    if (indices) primitive.indicesId = this.#loadAccessor(indices);

    // Load attributes
    gltfPrimitive.listSemantics().forEach((name) => {
      const attribute = gltfPrimitive.getAttribute(name);
      if (!attribute) return;

      switch (name) {
        case "POSITION":
          primitive.POSITION = this.#loadAccessor(attribute);
          break;
        case "NORMAL":
          primitive.NORMAL = this.#loadAccessor(attribute);
          break;
        case "TANGENT":
          primitive.TANGENT = this.#loadAccessor(attribute);
          break;
        case "TEXCOORD_0":
          primitive.TEXCOORD_0 = this.#loadAccessor(attribute);
          break;
        case "TEXCOORD_1":
          primitive.TEXCOORD_1 = this.#loadAccessor(attribute);
          break;
        case "COLOR_0":
          primitive.COLOR_0 = this.#loadAccessor(attribute);
          break;
        case "JOINTS_0":
          primitive.JOINTS_0 = this.#loadAccessor(attribute);
          break;
        case "WEIGHTS_0":
          primitive.WEIGHTS_0 = this.#loadAccessor(attribute);
          break;
        default:
          console.warn(`Attribute ${name} not supported`);
          return;
      }
    });

    // Load morph targets
    gltfPrimitive.listTargets().forEach((target) => {
      this.#loadMorphTarget(target, primitive);
    });

    return primitive;
  }

  #loadMorphTarget(target: IPrimitiveTarget, mesh: Primitive) {
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

    // Create node
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
    if (emissiveTexture && emissiveInfo) {
      materialState.emissiveTexture = this.#loadTexture(emissiveTexture, emissiveInfo);
    }

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
    if (normalTexture && normalInfo) {
      materialState.normalTexture = this.#loadTexture(normalTexture, normalInfo);
    }

    // Occlusion strength
    materialState.occlusionStrength = material.getOcclusionStrength();

    // Occlusion texture
    const occlusionTexture = material.getOcclusionTexture();
    const occlusionInfo = material.getOcclusionTextureInfo();
    if (occlusionTexture && occlusionInfo) {
      materialState.occlusionTexture = this.#loadTexture(occlusionTexture, occlusionInfo);
    }

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
    const array = texture.getImage();

    if (array) {
      const blob = new Blob([array], { type: mimeType });

      // Create image bitmap asynchronously
      const promise = createImageBitmap(blob).then((bitmap) => {
        const image = new Image({ id, array, bitmap, mimeType });
        image.isInternal = true;
        this.#scene.addImage(image);
      });

      this.#pending.push(promise);

      this.#map.images.set(texture, id);
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
    const type = accessor.getType();

    const accessorState = new Accessor({
      array,
      elementSize,
      type,
      normalized,
    });

    accessorState.isInternal = true;

    // Add to scene
    this.#scene.addAccessor(accessorState);

    this.#map.accessors.set(accessor, accessorState.id);
    return accessorState.id;
  }
}
