import { Document, GLTF, Material } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Vec3, Vec4 } from "../../types";
import { Scene } from "../Scene";
import { Attribute } from "./Attribute";
import { TextureInfoJSON, TextureInfoUtils } from "./TextureInfoUtils";
import { Textures } from "./Textures";

type TextureId = string;

export interface MaterialJSON {
  alphaMode: GLTF.MaterialAlphaMode;
  alphaCutoff: number;
  doubleSided: boolean;

  baseColorFactor: Vec4;
  baseColorTexture: TextureId | null;
  baseColorTextureInfo: TextureInfoJSON | null;

  emissiveFactor: Vec3;
  emissiveTexture: TextureId | null;
  emissiveTextureInfo: TextureInfoJSON | null;

  normalScale: number;
  normalTexture: TextureId | null;
  normalTextureInfo: TextureInfoJSON | null;

  occlusionStrength: number;
  occlusionTexture: TextureId | null;
  occlusionTextureInfo: TextureInfoJSON | null;

  roughnessFactor: number;
  metallicFactor: number;
  metallicRoughnessTexture: TextureId | null;
  metallicRoughnessTextureInfo: TextureInfoJSON | null;
}

/**
 * Stores and manages materials within a {@link Scene}.
 *
 * @group Scene
 */
export class Materials extends Attribute<Material, MaterialJSON> {
  #doc: Document;
  #texture: Textures;
  store = new Map<string, Material>();

  constructor(scene: Scene) {
    super();

    this.#doc = scene.doc;
    this.#texture = scene.texture;
  }

  getId(material: Material) {
    for (const [id, m] of this.store) {
      if (m === material) return id;
    }
  }

  create(json: Partial<MaterialJSON> = {}, id?: string) {
    const material = this.#doc.createMaterial();
    this.applyJSON(material, json);

    const { id: materialId } = this.process(material, id);

    this.emitCreate(materialId);

    return { id: materialId, object: material };
  }

  process(material: Material, id?: string) {
    const materialId = id ?? nanoid();
    this.store.set(materialId, material);

    material.addEventListener("dispose", () => {
      this.store.delete(materialId);
    });

    return { id: materialId };
  }

  processChanges() {
    const changed: Material[] = [];

    // Add new materials
    this.#doc
      .getRoot()
      .listMaterials()
      .forEach((material) => {
        const materialId = this.getId(material);
        if (materialId) return;

        this.process(material);
        changed.push(material);
      });

    return changed;
  }

  applyJSON(material: Material, json: Partial<MaterialJSON>) {
    if (json.alphaMode !== undefined) material.setAlphaMode(json.alphaMode);
    if (json.alphaCutoff !== undefined) material.setAlphaCutoff(json.alphaCutoff);
    if (json.doubleSided !== undefined) material.setDoubleSided(json.doubleSided);

    if (json.baseColorFactor !== undefined) material.setBaseColorFactor(json.baseColorFactor);
    if (json.baseColorTexture) {
      const texture = this.#texture.store.get(json.baseColorTexture);
      if (texture) material.setBaseColorTexture(texture);
    }
    if (json.baseColorTextureInfo) {
      const info = material.getBaseColorTextureInfo();
      if (info) TextureInfoUtils.applyJSON(info, json.baseColorTextureInfo);
    }

    if (json.emissiveFactor !== undefined) material.setEmissiveFactor(json.emissiveFactor);
    if (json.emissiveTexture) {
      const texture = this.#texture.store.get(json.emissiveTexture);
      if (texture) material.setEmissiveTexture(texture);
    }
    if (json.emissiveTextureInfo) {
      const info = material.getEmissiveTextureInfo();
      if (info) TextureInfoUtils.applyJSON(info, json.emissiveTextureInfo);
    }

    if (json.normalScale !== undefined) material.setNormalScale(json.normalScale);
    if (json.normalTexture) {
      const texture = this.#texture.store.get(json.normalTexture);
      if (texture) material.setNormalTexture(texture);
    }
    if (json.normalTextureInfo) {
      const info = material.getNormalTextureInfo();
      if (info) TextureInfoUtils.applyJSON(info, json.normalTextureInfo);
    }

    if (json.occlusionStrength !== undefined) material.setOcclusionStrength(json.occlusionStrength);
    if (json.occlusionTexture) {
      const texture = this.#texture.store.get(json.occlusionTexture);
      if (texture) material.setOcclusionTexture(texture);
    }
    if (json.occlusionTextureInfo) {
      const info = material.getOcclusionTextureInfo();
      if (info) TextureInfoUtils.applyJSON(info, json.occlusionTextureInfo);
    }

    if (json.roughnessFactor !== undefined) material.setRoughnessFactor(json.roughnessFactor);
    if (json.metallicFactor !== undefined) material.setMetallicFactor(json.metallicFactor);
    if (json.metallicRoughnessTexture) {
      const texture = this.#texture.store.get(json.metallicRoughnessTexture);
      if (texture) material.setMetallicRoughnessTexture(texture);
    }
    if (json.metallicRoughnessTextureInfo) {
      const info = material.getMetallicRoughnessTextureInfo();
      if (info) TextureInfoUtils.applyJSON(info, json.metallicRoughnessTextureInfo);
    }
  }

  toJSON(material: Material): MaterialJSON {
    const baseColorTexture = material.getBaseColorTexture();
    const baseColorTextureId = baseColorTexture ? this.#texture.getId(baseColorTexture) : null;
    if (baseColorTextureId === undefined) throw new Error("Base color texture not found");

    const baseColorInfo = material.getBaseColorTextureInfo();
    const baseColorInfoJSON = baseColorInfo ? TextureInfoUtils.toJSON(baseColorInfo) : null;

    const emissiveTexture = material.getEmissiveTexture();
    const emissiveTextureId = emissiveTexture ? this.#texture.getId(emissiveTexture) : null;
    if (emissiveTextureId === undefined) throw new Error("Emissive texture not found");

    const emissiveInfo = material.getEmissiveTextureInfo();
    const emissiveInfoJSON = emissiveInfo ? TextureInfoUtils.toJSON(emissiveInfo) : null;

    const normalTexture = material.getNormalTexture();
    const normalTextureId = normalTexture ? this.#texture.getId(normalTexture) : null;
    if (normalTextureId === undefined) throw new Error("Normal texture not found");

    const normalInfo = material.getNormalTextureInfo();
    const normalInfoJSON = normalInfo ? TextureInfoUtils.toJSON(normalInfo) : null;

    const occlusionTexture = material.getOcclusionTexture();
    const occlusionTextureId = occlusionTexture ? this.#texture.getId(occlusionTexture) : null;
    if (occlusionTextureId === undefined) throw new Error("Occlusion texture not found");

    const occlusionInfo = material.getOcclusionTextureInfo();
    const occlusionInfoJSON = occlusionInfo ? TextureInfoUtils.toJSON(occlusionInfo) : null;

    const metallicRoughnessTexture = material.getMetallicRoughnessTexture();
    const metallicRoughnessTextureId = metallicRoughnessTexture
      ? this.#texture.getId(metallicRoughnessTexture)
      : null;
    if (metallicRoughnessTextureId === undefined)
      throw new Error("Metallic roughness texture not found");

    const metallicRoughnessInfo = material.getMetallicRoughnessTextureInfo();
    const metallicRoughnessInfoJSON = metallicRoughnessInfo
      ? TextureInfoUtils.toJSON(metallicRoughnessInfo)
      : null;

    return {
      alphaMode: material.getAlphaMode(),
      alphaCutoff: material.getAlphaCutoff(),
      doubleSided: material.getDoubleSided(),

      baseColorFactor: material.getBaseColorFactor(),
      baseColorTexture: baseColorTextureId,
      baseColorTextureInfo: baseColorInfoJSON,

      emissiveFactor: material.getEmissiveFactor(),
      emissiveTexture: emissiveTextureId,
      emissiveTextureInfo: emissiveInfoJSON,

      normalScale: material.getNormalScale(),
      normalTexture: normalTextureId,
      normalTextureInfo: normalInfoJSON,

      occlusionStrength: material.getOcclusionStrength(),
      occlusionTexture: occlusionTextureId,
      occlusionTextureInfo: occlusionInfoJSON,

      roughnessFactor: material.getRoughnessFactor(),
      metallicFactor: material.getMetallicFactor(),
      metallicRoughnessTexture: metallicRoughnessTextureId,
      metallicRoughnessTextureInfo: metallicRoughnessInfoJSON,
    };
  }
}
