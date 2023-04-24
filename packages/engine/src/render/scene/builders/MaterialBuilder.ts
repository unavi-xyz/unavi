import { Material, TextureInfo } from "@gltf-transform/core";
import { Transform } from "@gltf-transform/extensions";
import {
  ClampToEdgeWrapping,
  DoubleSide,
  FrontSide,
  LinearFilter,
  LinearMipMapLinearFilter,
  LinearMipMapNearestFilter,
  MeshStandardMaterial,
  MirroredRepeatWrapping,
  NearestFilter,
  NearestMipMapLinearFilter,
  NearestMipMapNearestFilter,
  RepeatWrapping,
  sRGBEncoding,
  Texture,
} from "three";

import { MaterialJSON } from "../../../scene";
import { subscribe } from "../../../utils/subscribe";
import { WEBGL_CONSTANTS } from "../../constants";
import { Builder } from "./Builder";

/**
 * @internal
 * Handles the conversion of materials to Three.js objects.
 */
export class MaterialBuilder extends Builder<Material, MaterialJSON, MeshStandardMaterial> {
  add(json: Partial<MaterialJSON>, id: string) {
    const previousObject = this.getObject(id);
    if (previousObject) throw new Error(`Material with id ${id} already exists.`);

    const { object: material } = this.scene.material.create(json, id);

    const object = new MeshStandardMaterial();
    this.setObject(id, object);

    subscribe(material, "Name", (value) => {
      object.name = value;
    });

    subscribe(material, "DoubleSided", (value) => {
      object.side = value ? DoubleSide : FrontSide;
    });

    subscribe(material, "BaseColorFactor", (value) => {
      object.color.fromArray(value);
    });

    subscribe(material, "MetallicFactor", (value) => {
      object.metalness = value;
    });

    subscribe(material, "RoughnessFactor", (value) => {
      object.roughness = value;
    });

    subscribe(material, "NormalScale", (value) => {
      object.normalScale.set(value, value);
    });

    subscribe(material, "OcclusionStrength", (value) => {
      object.aoMapIntensity = value;
    });

    subscribe(material, "EmissiveFactor", (value) => {
      object.emissive.fromArray(value);
    });

    subscribe(material, "Alpha", (value) => {
      object.transparent = value < 1;
      object.opacity = value;
    });

    subscribe(material, "AlphaMode", (mode) => {
      return subscribe(material, "AlphaCutoff", (cutoff) => {
        switch (mode) {
          case "OPAQUE": {
            object.transparent = false;
            object.depthWrite = true;
            object.alphaTest = 0;
            break;
          }

          case "MASK": {
            object.transparent = false;
            object.depthWrite = true;
            object.alphaTest = cutoff;
            break;
          }

          case "BLEND": {
            object.transparent = true;
            object.depthWrite = false;
            object.alphaTest = 0;
            break;
          }
        }
      });
    });

    subscribe(material, "BaseColorTexture", (texture) => {
      if (!texture) {
        object.map = null;
        return;
      }

      const textureId = this.scene.texture.getId(texture);
      if (!textureId) throw new Error("Texture id not found.");

      const textureObject = this.scene.builders.texture.getObject(textureId);
      if (!textureObject) throw new Error("Texture object not found.");

      object.map = textureObject;
      textureObject.encoding = sRGBEncoding;

      return subscribe(material, "BaseColorTextureInfo", (info) => {
        applyTextureInfo(textureObject, info);
      });
    });

    subscribe(material, "MetallicRoughnessTexture", (texture) => {
      if (!texture) {
        object.metalnessMap = null;
        object.roughnessMap = null;
        return;
      }

      const textureId = this.scene.texture.getId(texture);
      if (!textureId) throw new Error("Texture id not found.");

      const textureObject = this.scene.builders.texture.getObject(textureId);
      if (!textureObject) throw new Error("Texture object not found.");

      object.metalnessMap = textureObject;
      object.roughnessMap = textureObject;

      return subscribe(material, "MetallicRoughnessTextureInfo", (info) => {
        applyTextureInfo(textureObject, info);
      });
    });

    subscribe(material, "NormalTexture", (texture) => {
      if (!texture) {
        object.normalMap = null;
        return;
      }

      const textureId = this.scene.texture.getId(texture);
      if (!textureId) throw new Error("Texture id not found.");

      const textureObject = this.scene.builders.texture.getObject(textureId);
      if (!textureObject) throw new Error("Texture object not found.");

      object.normalMap = textureObject;

      return subscribe(material, "NormalTextureInfo", (info) => {
        applyTextureInfo(textureObject, info);
      });
    });

    subscribe(material, "OcclusionTexture", (texture) => {
      if (!texture) {
        object.aoMap = null;
        return;
      }

      const textureId = this.scene.texture.getId(texture);
      if (!textureId) throw new Error("Texture id not found.");

      const textureObject = this.scene.builders.texture.getObject(textureId);
      if (!textureObject) throw new Error("Texture object not found.");

      object.aoMap = textureObject;

      return subscribe(material, "OcclusionTextureInfo", (info) => {
        applyTextureInfo(textureObject, info);
      });
    });

    subscribe(material, "EmissiveTexture", (texture) => {
      if (!texture) {
        object.emissiveMap = null;
        return;
      }

      const textureId = this.scene.texture.getId(texture);
      if (!textureId) throw new Error("Texture id not found.");

      const textureObject = this.scene.builders.texture.getObject(textureId);
      if (!textureObject) throw new Error("Texture object not found.");

      object.emissiveMap = textureObject;
      textureObject.encoding = sRGBEncoding;

      return subscribe(material, "EmissiveTextureInfo", (info) => {
        applyTextureInfo(textureObject, info);
      });
    });

    material.addEventListener("dispose", () => {
      object.dispose();
      this.setObject(id, null);
    });

    return material;
  }

  remove(id: string) {
    this.scene.material.store.get(id)?.dispose();
  }

  update(json: Partial<MaterialJSON>, id: string) {
    const material = this.scene.material.store.get(id);
    if (!material) throw new Error(`Material with id ${id} does not exist.`);

    this.scene.material.applyJSON(material, json);
  }
}

function applyTextureInfo(textureObject: Texture, info: TextureInfo | null) {
  switch (info?.getMagFilter()) {
    case WEBGL_CONSTANTS.NEAREST: {
      textureObject.magFilter = NearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR:
    default: {
      textureObject.magFilter = LinearFilter;
      break;
    }
  }

  switch (info?.getMinFilter()) {
    case WEBGL_CONSTANTS.NEAREST: {
      textureObject.minFilter = NearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.NEAREST_MIPMAP_NEAREST: {
      textureObject.minFilter = NearestMipMapNearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.NEAREST_MIPMAP_LINEAR: {
      textureObject.minFilter = NearestMipMapLinearFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR_MIPMAP_NEAREST: {
      textureObject.minFilter = LinearMipMapNearestFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR_MIPMAP_LINEAR: {
      textureObject.minFilter = LinearMipMapLinearFilter;
      break;
    }

    case WEBGL_CONSTANTS.LINEAR:
    default: {
      textureObject.minFilter = LinearFilter;
      break;
    }
  }

  switch (info?.getWrapS()) {
    case WEBGL_CONSTANTS.CLAMP_TO_EDGE: {
      textureObject.wrapS = ClampToEdgeWrapping;
      break;
    }

    case WEBGL_CONSTANTS.MIRRORED_REPEAT: {
      textureObject.wrapS = MirroredRepeatWrapping;
      break;
    }

    case WEBGL_CONSTANTS.REPEAT:
    default: {
      textureObject.wrapS = RepeatWrapping;
      break;
    }
  }

  switch (info?.getWrapT()) {
    case WEBGL_CONSTANTS.CLAMP_TO_EDGE: {
      textureObject.wrapT = ClampToEdgeWrapping;
      break;
    }

    case WEBGL_CONSTANTS.MIRRORED_REPEAT: {
      textureObject.wrapT = MirroredRepeatWrapping;
      break;
    }

    case WEBGL_CONSTANTS.REPEAT:
    default: {
      textureObject.wrapT = RepeatWrapping;
      break;
    }
  }

  // Texture transform
  const transform = info?.getExtension<Transform>("KHR_texture_transform");
  if (transform) {
    textureObject.offset.fromArray(transform.getOffset());
    textureObject.repeat.fromArray(transform.getScale());
    textureObject.rotation = transform.getRotation();
  } else {
    textureObject.offset.set(0, 0);
    textureObject.repeat.set(1, 1);
    textureObject.rotation = 0;
  }
}
