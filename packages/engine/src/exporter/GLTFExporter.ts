import {
  Accessor as IAccessor,
  Document,
  Material as IMaterial,
  Mesh as IMesh,
  Node as INode,
  Texture as ITexture,
  WebIO,
} from "@gltf-transform/core";
import { dedup, prune } from "@gltf-transform/functions";

import { extensions } from "../gltf/constants";
import { ColliderExtension } from "../gltf/extensions/Collider/ColliderExtension";
import { SpawnPointExtension } from "../gltf/extensions/SpawnPoint/SpawnPointExtension";
import { RenderExport } from "../render/types";
import {
  Accessor,
  Animation,
  Material,
  Node,
  Scene,
  SceneJSON,
  Texture,
} from "../scene";
import { setTextureInfo } from "./utils/setTextureInfo";

/*
 * Exports the scene as a glTF file.
 */
export class GLTFExporter {
  #doc = new Document();
  #extensions = {
    collider: this.#doc.createExtension(ColliderExtension),
    spawnPoint: this.#doc.createExtension(SpawnPointExtension),
  };

  #scene = new Scene();
  #gltfScene = this.#doc.createScene("Scene");
  #renderData: RenderExport | null = null;

  #buffer = this.#doc.createBuffer();

  #cache = {
    accessors: new Map<string, IAccessor>(),
    meshes: new Map<string, IMesh>(),
    materials: new Map<string, IMaterial>(),
    nodes: new Map<string, INode>(),
  };

  constructor() {
    this.#doc.getRoot().setDefaultScene(this.#gltfScene);
  }

  async parse(json: SceneJSON, renderData?: RenderExport) {
    this.#scene.loadJSON(json);

    if (renderData) this.#renderData = renderData;

    // Parse spawn
    if (this.#scene.spawn) {
      const spawn = this.#extensions.spawnPoint.createSpawnPoint();
      const spawnNode = this.#doc.createNode("Spawn");
      spawnNode.setTranslation(this.#scene.spawn);
      spawnNode.setExtension(spawn.extensionName, spawn);
      this.#gltfScene.addChild(spawnNode);
    }

    // Parse accessors
    Object.values(this.#scene.accessors).forEach((accessor) =>
      this.#parseAccessor(accessor)
    );

    // Parse materials
    Object.values(this.#scene.materials).forEach((material) =>
      this.#parseMaterial(material)
    );

    // Parse meshes
    Object.values(this.#scene.meshes).forEach((mesh) =>
      this.#parseMesh(mesh.id)
    );

    // Parse nodes
    const rootChildren = Object.values(this.#scene.nodes).filter(
      (e) => e.parentId === "root"
    );
    rootChildren.forEach((node) => this.#parseNode(node));

    // Parse skins
    Object.values(this.#scene.nodes).forEach((node) => {
      const mesh = node.meshId ? this.#scene.meshes[node.meshId] : null;
      if (!mesh || mesh.type !== "Primitives") return;

      const gltfNode = this.#cache.nodes.get(node.id);
      if (!gltfNode) throw new Error("Node not found");

      mesh.primitives.forEach((primitive) => {
        if (!primitive.skin) return;

        // Create skin
        const skin = this.#doc.createSkin();
        gltfNode.setSkin(skin);

        // Set inverse bind matrices
        const accessor = this.#cache.accessors.get(
          primitive.skin.inverseBindMatricesId
        );
        if (!accessor) throw new Error("Accessor not found");
        skin.setInverseBindMatrices(accessor);

        // Set joints
        primitive.skin.jointIds.forEach((jointId) => {
          const jointNode = this.#cache.nodes.get(jointId);
          if (!jointNode) throw new Error("Node not found");
          skin.addJoint(jointNode);
        });
      });
    });

    // Parse animations
    Object.values(this.#scene.animations).forEach((animation) =>
      this.#parseAnimation(animation)
    );

    // Apply transforms
    await this.#doc.transform(dedup(), prune());

    // Write to binary
    const io = new WebIO().registerExtensions(extensions);
    const glb = await io.writeBinary(this.#doc);

    return glb;
  }

  #parseAnimation(animation: Animation) {
    const gltfAnimation = this.#doc.createAnimation(animation.name);

    animation.channels.forEach((channel) => {
      const input = this.#cache.accessors.get(channel.sampler.inputId);
      const output = this.#cache.accessors.get(channel.sampler.outputId);
      if (!input || !output) throw new Error("Invalid animation channel");

      const sampler = this.#doc.createAnimationSampler();
      sampler.setInput(input);
      sampler.setOutput(output);

      const targetNode = this.#cache.nodes.get(channel.targetId);
      if (!targetNode) throw new Error("Target not found");

      const gltfChannel = this.#doc.createAnimationChannel();
      gltfChannel.setSampler(sampler);
      gltfChannel.setTargetNode(targetNode);
      if (channel.path) gltfChannel.setTargetPath(channel.path);

      gltfAnimation.addSampler(sampler);
      gltfAnimation.addChannel(gltfChannel);
    });
  }

  #parseNode(node: Node) {
    // Create node
    const gltfNode = this.#doc.createNode(node.name);
    this.#cache.nodes.set(node.id, gltfNode);

    // Add to scene
    this.#gltfScene.addChild(gltfNode);

    // Set transform
    gltfNode.setTranslation(node.position);
    gltfNode.setRotation(node.rotation);
    gltfNode.setScale(node.scale);

    // Parse mesh
    const mesh = node.meshId ? this.#cache.meshes.get(node.meshId) : null;
    if (mesh) gltfNode.setMesh(mesh);

    // Add to parent
    const parent =
      node.parentId !== "root" ? this.#cache.nodes.get(node.parentId) : null;
    if (parent) parent.addChild(gltfNode);

    // Set collider
    if (node.collider) {
      const collider = this.#extensions.collider.createCollider();
      collider.setType(node.collider.type);

      switch (node.collider.type) {
        case "box": {
          collider.setSize(node.collider.size);
          break;
        }

        case "sphere": {
          collider.setRadius(node.collider.radius);
          break;
        }

        case "cylinder": {
          collider.setRadius(node.collider.radius);
          collider.setHeight(node.collider.height);
          break;
        }

        case "hull":
        case "mesh": {
          if (!node.collider.meshId) break;

          const colliderMesh = this.#cache.meshes.get(node.collider.meshId);
          if (!colliderMesh) throw new Error("Mesh not found");

          collider.setMesh(colliderMesh);
          break;
        }
      }

      gltfNode.setExtension(collider.extensionName, collider);
    }

    // Parse children
    node.children.forEach((child) => this.#parseNode(child));
  }

  #parseMesh(meshId: string): IMesh {
    const mesh = this.#scene.meshes[meshId];
    if (!mesh) throw new Error("Mesh not found");

    // Create mesh
    const gltfMesh = this.#doc.createMesh(mesh.name ?? mesh.type);

    // Set attributes
    switch (mesh.type) {
      case "Box":
      case "Cylinder":
      case "Sphere": {
        // Create primitive
        const gltfPrimitive = this.#doc.createPrimitive();
        gltfMesh.addPrimitive(gltfPrimitive);

        // Set material
        if (mesh.materialId) {
          const material = this.#cache.materials.get(mesh.materialId);
          if (!material) throw new Error("Material not found");
          gltfPrimitive.setMaterial(material);
        }

        // Set mode
        gltfPrimitive.setMode(4); // TRIANGLES

        // Set attributes
        this.#renderData?.forEach(
          ({ nodeId, attributeName, array, normalized, type }) => {
            if (nodeId !== mesh.id) return;

            const gltfAccessor = this.#doc.createAccessor();

            gltfAccessor.setArray(array);
            gltfAccessor.setNormalized(normalized);
            gltfAccessor.setType(type);
            gltfAccessor.setBuffer(this.#buffer);

            if (attributeName === "indices")
              gltfPrimitive.setIndices(gltfAccessor);
            else gltfPrimitive.setAttribute(attributeName, gltfAccessor);
          }
        );

        break;
      }

      case "Primitives": {
        mesh.primitives.forEach((primitive) => {
          // Create primitive
          const gltfPrimitive = this.#doc.createPrimitive();
          gltfMesh.addPrimitive(gltfPrimitive);

          // Set material
          if (primitive.materialId) {
            const material = this.#cache.materials.get(primitive.materialId);
            if (!material) throw new Error("Material not found");
            gltfPrimitive.setMaterial(material);
          }

          // Set mode
          gltfPrimitive.setMode(primitive.mode);

          // Set weights
          gltfMesh.setWeights(primitive.weights);

          if (primitive.indicesId) {
            const accessor = this.#cache.accessors.get(primitive.indicesId);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setIndices(accessor);
          }

          if (primitive.POSITION) {
            const accessor = this.#cache.accessors.get(primitive.POSITION);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("POSITION", accessor);
          }

          if (primitive.NORMAL) {
            const accessor = this.#cache.accessors.get(primitive.NORMAL);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("NORMAL", accessor);
          }

          if (primitive.TANGENT) {
            const accessor = this.#cache.accessors.get(primitive.TANGENT);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("TANGENT", accessor);
          }

          if (primitive.TEXCOORD_0) {
            const accessor = this.#cache.accessors.get(primitive.TEXCOORD_0);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("TEXCOORD_0", accessor);
          }

          if (primitive.TEXCOORD_1) {
            const accessor = this.#cache.accessors.get(primitive.TEXCOORD_1);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("TEXCOORD_1", accessor);
          }

          if (primitive.COLOR_0) {
            const accessor = this.#cache.accessors.get(primitive.COLOR_0);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("COLOR_0", accessor);
          }

          if (primitive.JOINTS_0) {
            const accessor = this.#cache.accessors.get(primitive.JOINTS_0);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("JOINTS_0", accessor);
          }

          if (primitive.WEIGHTS_0) {
            const accessor = this.#cache.accessors.get(primitive.WEIGHTS_0);
            if (!accessor) throw new Error("Accessor not found");
            gltfPrimitive.setAttribute("WEIGHTS_0", accessor);
          }

          if (primitive.morphPositionIds) {
            primitive.morphPositionIds.forEach((morphId) => {
              const morphPosition = this.#cache.accessors.get(morphId);
              if (!morphPosition) throw new Error("Accessor not found");

              const primitiveTarget =
                this.#doc.createPrimitiveTarget("POSITION");
              primitiveTarget.setAttribute("POSITION", morphPosition);
              gltfPrimitive.addTarget(primitiveTarget);
            });
          }

          if (primitive.morphNormalIds) {
            primitive.morphNormalIds.forEach((morphId) => {
              const morphNormal = this.#cache.accessors.get(morphId);
              if (!morphNormal) throw new Error("Accessor not found");

              const primitiveTarget = this.#doc.createPrimitiveTarget("NORMAL");
              primitiveTarget.setAttribute("NORMAL", morphNormal);
              gltfPrimitive.addTarget(primitiveTarget);
            });
          }

          if (primitive.morphTangentIds) {
            primitive.morphTangentIds.forEach((morphId) => {
              const morphTangent = this.#cache.accessors.get(morphId);
              if (!morphTangent) throw new Error("Accessor not found");

              const primitiveTarget =
                this.#doc.createPrimitiveTarget("TANGENT");
              primitiveTarget.setAttribute("TANGENT", morphTangent);
              gltfPrimitive.addTarget(primitiveTarget);
            });
          }
        });
        break;
      }
    }

    this.#cache.meshes.set(mesh.id, gltfMesh);

    return gltfMesh;
  }

  #parseMaterial(material: Material) {
    const gltfMaterial = this.#doc.createMaterial(material.name);

    gltfMaterial.setDoubleSided(material.doubleSided);
    gltfMaterial.setBaseColorFactor(material.color);

    gltfMaterial.setMetallicFactor(material.metalness);
    gltfMaterial.setRoughnessFactor(material.roughness);

    gltfMaterial.setEmissiveFactor(material.emissive);
    gltfMaterial.setNormalScale(material.normalScale);
    gltfMaterial.setOcclusionStrength(material.occlusionStrength);

    gltfMaterial.setAlpha(material.alpha);
    gltfMaterial.setAlphaCutoff(material.alphaCutoff);
    gltfMaterial.setAlphaMode(material.alphaMode);

    if (material.colorTexture) {
      gltfMaterial.setBaseColorTexture(
        this.#parseTexture(material.colorTexture)
      );
      const info = gltfMaterial.getBaseColorTextureInfo();
      setTextureInfo(info, material.colorTexture);
    }

    if (material.emissiveTexture) {
      gltfMaterial.setEmissiveTexture(
        this.#parseTexture(material.emissiveTexture)
      );
      const info = gltfMaterial.getEmissiveTextureInfo();
      setTextureInfo(info, material.emissiveTexture);
    }

    if (material.normalTexture) {
      gltfMaterial.setNormalTexture(this.#parseTexture(material.normalTexture));
      const info = gltfMaterial.getNormalTextureInfo();
      setTextureInfo(info, material.normalTexture);
    }

    if (material.occlusionTexture) {
      gltfMaterial.setOcclusionTexture(
        this.#parseTexture(material.occlusionTexture)
      );
      const info = gltfMaterial.getOcclusionTextureInfo();
      setTextureInfo(info, material.occlusionTexture);
    }

    if (material.metallicRoughnessTexture) {
      gltfMaterial.setMetallicRoughnessTexture(
        this.#parseTexture(material.metallicRoughnessTexture)
      );
      const info = gltfMaterial.getMetallicRoughnessTextureInfo();
      setTextureInfo(info, material.metallicRoughnessTexture);
    }

    this.#cache.materials.set(material.id, gltfMaterial);
  }

  #parseTexture(texture: Texture): ITexture {
    const gltfTexture = this.#doc.createTexture();

    if (texture.imageId) {
      const image = this.#scene.images[texture.imageId];
      if (!image) throw new Error("Image not found");

      gltfTexture.setImage(image.array);
      gltfTexture.setMimeType(image.mimeType);
    }

    return gltfTexture;
  }

  #parseAccessor(accessor: Accessor) {
    const gltfAccessor = this.#doc.createAccessor();

    gltfAccessor.setArray(accessor.array);
    gltfAccessor.setNormalized(accessor.normalized);
    gltfAccessor.setType(accessor.type);
    gltfAccessor.setBuffer(this.#buffer);

    this.#cache.accessors.set(accessor.id, gltfAccessor);
  }
}
