import {
  Accessor as gltfAccessor,
  Document,
  Material as gltfMaterial,
  Node,
  Texture as gltfTexture,
  WebIO,
} from "@gltf-transform/core";
import { dedup, prune } from "@gltf-transform/functions";

import { RenderExport } from "../render/types";
import {
  Accessor,
  Animation,
  Entity,
  Material,
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
  #scene = new Scene();
  #gltfScene = this.#doc.createScene("Scene");
  #renderData: RenderExport | null = null;

  #buffer = this.#doc.createBuffer();

  #cache = {
    accessors: new Map<string, gltfAccessor>(),
    materials: new Map<string, gltfMaterial>(),
    entities: new Map<string, Node>(),
  };

  constructor() {
    this.#doc.getRoot().setDefaultScene(this.#gltfScene);
  }

  async parse(json: SceneJSON, renderData?: RenderExport) {
    this.#scene.loadJSON(json);

    if (renderData) this.#renderData = renderData;

    // Parse accessors
    Object.values(this.#scene.accessors).forEach((accessor) =>
      this.#parseAccessor(accessor)
    );

    // Parse materials
    Object.values(this.#scene.materials).forEach((material) =>
      this.#parseMaterial(material)
    );

    // Parse entities
    const rootChildren = Object.values(this.#scene.entities).filter(
      (e) => e.parentId === "root"
    );
    rootChildren.forEach((entity) => this.#parseEntity(entity, null));

    // Parse skins
    Object.values(this.#scene.entities).forEach((entity) => {
      if (!entity.mesh || entity.mesh.type !== "Primitive" || !entity.mesh.skin)
        return;

      const node = this.#cache.entities.get(entity.id);
      if (!node) throw new Error("Node not found");

      const skin = this.#doc.createSkin();
      node.setSkin(skin);

      // Set inverse bind matrices
      const accessor = this.#cache.accessors.get(
        entity.mesh.skin.inverseBindMatricesId
      );
      if (!accessor) throw new Error("Accessor not found");
      skin.setInverseBindMatrices(accessor);

      // Set joints
      entity.mesh.skin.jointIds.forEach((jointId) => {
        const jointNode = this.#cache.entities.get(jointId);
        if (!jointNode) throw new Error("Node not found");
        skin.addJoint(jointNode);
      });
    });

    // Parse animations
    Object.values(this.#scene.animations).forEach((animation) =>
      this.#parseAnimation(animation)
    );

    // Apply transforms
    await this.#doc.transform(dedup(), prune());

    // Write to binary
    const io = new WebIO();
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

      const targetNode = this.#cache.entities.get(channel.targetId);
      if (!targetNode) throw new Error("Target not found");

      const gltfChannel = this.#doc.createAnimationChannel();
      gltfChannel.setSampler(sampler);
      gltfChannel.setTargetNode(targetNode);
      if (channel.path) gltfChannel.setTargetPath(channel.path);

      gltfAnimation.addSampler(sampler);
      gltfAnimation.addChannel(gltfChannel);
    });
  }

  #parseEntity(entity: Entity, parent: Node | null) {
    // Create node
    const node = this.#doc.createNode(entity.name);

    // Add to scene
    this.#gltfScene.addChild(node);

    // Set transform
    node.setTranslation(entity.position);
    node.setRotation(entity.rotation);
    node.setScale(entity.scale);

    // Parse mesh
    this.#parseMesh(entity, node);

    // Parse children
    entity.children.forEach((child) => this.#parseEntity(child, node));

    // Add to parent
    if (parent) parent.addChild(node);

    this.#cache.entities.set(entity.id, node);
  }

  #parseMesh(entity: Entity, node: Node) {
    if (!entity.mesh || entity.mesh.type === "glTF") return;

    // Create mesh
    const mesh = this.#doc.createMesh(entity.mesh.type);
    node.setMesh(mesh);

    // Create primitive
    const primitive = this.#doc.createPrimitive();
    mesh.addPrimitive(primitive);

    // Set material
    if (entity.materialId) {
      const material = this.#cache.materials.get(entity.materialId);
      if (!material) throw new Error("Material not found");
      primitive.setMaterial(material);
    }

    // Set attributes
    switch (entity.mesh.type) {
      case "Box":
      case "Cylinder":
      case "Sphere": {
        primitive.setMode(4); // TRIANGLES

        if (!this.#renderData) throw new Error("Render data not found");

        this.#renderData.forEach(
          ({ entityId, attributeName, array, normalized, type }) => {
            if (entityId !== entity.id) return;

            const gltfAccessor = this.#doc.createAccessor();

            gltfAccessor.setArray(array);
            gltfAccessor.setNormalized(normalized);
            gltfAccessor.setType(type);
            gltfAccessor.setBuffer(this.#buffer);

            if (attributeName === "indices") primitive.setIndices(gltfAccessor);
            else primitive.setAttribute(attributeName, gltfAccessor);
          }
        );

        break;
      }
      case "Primitive": {
        primitive.setMode(entity.mesh.mode);

        mesh.setWeights(entity.mesh.weights);

        if (entity.mesh.indicesId) {
          const accessor = this.#cache.accessors.get(entity.mesh.indicesId);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setIndices(accessor);
        }

        if (entity.mesh.POSITION) {
          const accessor = this.#cache.accessors.get(entity.mesh.POSITION);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("POSITION", accessor);
        }

        if (entity.mesh.NORMAL) {
          const accessor = this.#cache.accessors.get(entity.mesh.NORMAL);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("NORMAL", accessor);
        }

        if (entity.mesh.TANGENT) {
          const accessor = this.#cache.accessors.get(entity.mesh.TANGENT);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("TANGENT", accessor);
        }

        if (entity.mesh.TEXCOORD_0) {
          const accessor = this.#cache.accessors.get(entity.mesh.TEXCOORD_0);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("TEXCOORD_0", accessor);
        }

        if (entity.mesh.TEXCOORD_1) {
          const accessor = this.#cache.accessors.get(entity.mesh.TEXCOORD_1);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("TEXCOORD_1", accessor);
        }

        if (entity.mesh.COLOR_0) {
          const accessor = this.#cache.accessors.get(entity.mesh.COLOR_0);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("COLOR_0", accessor);
        }

        if (entity.mesh.JOINTS_0) {
          const accessor = this.#cache.accessors.get(entity.mesh.JOINTS_0);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("JOINTS_0", accessor);
        }

        if (entity.mesh.WEIGHTS_0) {
          const accessor = this.#cache.accessors.get(entity.mesh.WEIGHTS_0);
          if (!accessor) throw new Error("Accessor not found");
          primitive.setAttribute("WEIGHTS_0", accessor);
        }

        if (entity.mesh.morphPositionIds) {
          entity.mesh.morphPositionIds.forEach((morphId) => {
            const morphPosition = this.#cache.accessors.get(morphId);
            if (!morphPosition) throw new Error("Accessor not found");

            const primitiveTarget = this.#doc.createPrimitiveTarget("POSITION");
            primitiveTarget.setAttribute("POSITION", morphPosition);
            primitive.addTarget(primitiveTarget);
          });
        }

        if (entity.mesh.morphNormalIds) {
          entity.mesh.morphNormalIds.forEach((morphId) => {
            const morphNormal = this.#cache.accessors.get(morphId);
            if (!morphNormal) throw new Error("Accessor not found");

            const primitiveTarget = this.#doc.createPrimitiveTarget("NORMAL");
            primitiveTarget.setAttribute("NORMAL", morphNormal);
            primitive.addTarget(primitiveTarget);
          });
        }

        if (entity.mesh.morphTangentIds) {
          entity.mesh.morphTangentIds.forEach((morphId) => {
            const morphTangent = this.#cache.accessors.get(morphId);
            if (!morphTangent) throw new Error("Accessor not found");

            const primitiveTarget = this.#doc.createPrimitiveTarget("TANGENT");
            primitiveTarget.setAttribute("TANGENT", morphTangent);
            primitive.addTarget(primitiveTarget);
          });
        }

        break;
      }
    }
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

  #parseTexture(texture: Texture): gltfTexture {
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
