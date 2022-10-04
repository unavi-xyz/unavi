import {
  Accessor as gltfAccessor,
  Document,
  GLTF,
  Material as gltfMaterial,
  Node,
  Texture as gltfTexture,
  WebIO,
} from "@gltf-transform/core";
import { center, dedup, prune } from "@gltf-transform/functions";

import {
  Accessor,
  Entity,
  Material,
  Scene,
  SceneJSON,
  Texture,
} from "../scene";
import { bitmapToArray } from "./utils/bitmapToArray";

/*
 * Exports the scene as a glTF file.
 */
export class GLTFExporter {
  #doc = new Document();
  #scene = new Scene();
  #gltfScene = this.#doc.createScene("Scene");

  #buffer = this.#doc.createBuffer();

  #cache = {
    accessors: new Map<string, gltfAccessor>(),
    materials: new Map<string, gltfMaterial>(),
  };

  constructor() {
    this.#doc.getRoot().setDefaultScene(this.#gltfScene);
  }

  async parse(json: SceneJSON) {
    this.#scene.loadJSON(json);

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

    // Apply transforms
    await this.#doc.transform(dedup(), prune(), center());

    // Write to binary
    const io = new WebIO();
    const glb = await io.writeBinary(this.#doc);

    // TODO: Get export working - currently pruning entire scene

    return glb;
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

        // TODO get indices + attributes from three.js

        break;
      }
      case "Primitive": {
        primitive.setMode(entity.mesh.mode as GLTF.MeshPrimitiveMode);

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
    }

    if (material.emissiveTexture) {
      gltfMaterial.setEmissiveTexture(
        this.#parseTexture(material.emissiveTexture)
      );
    }

    if (material.normalTexture) {
      gltfMaterial.setNormalTexture(this.#parseTexture(material.normalTexture));
    }

    if (material.occlusionTexture) {
      gltfMaterial.setOcclusionTexture(
        this.#parseTexture(material.occlusionTexture)
      );
    }

    if (material.metallicRoughnessTexture) {
      gltfMaterial.setMetallicRoughnessTexture(
        this.#parseTexture(material.metallicRoughnessTexture)
      );
    }

    this.#cache.materials.set(material.id, gltfMaterial);
  }

  #parseTexture(texture: Texture): gltfTexture {
    const gltfTexture = this.#doc.createTexture();

    if (texture.imageId) {
      const image = this.#scene.images[texture.imageId];
      if (!image) throw new Error("Image not found");

      const array = bitmapToArray(image.bitmap);
      gltfTexture.setImage(array);
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
