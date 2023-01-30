import { GLTF, TextureInfo } from "@gltf-transform/core";
import { Transform } from "@gltf-transform/extensions";

import { Vec2 } from "../../types";

export interface TextureInfoJSON {
  texCoord: number;
  magFilter: GLTF.TextureMagFilter | null;
  minFilter: GLTF.TextureMinFilter | null;
  wrapS: GLTF.TextureWrapMode;
  wrapT: GLTF.TextureWrapMode;
  offset: Vec2 | null;
  rotation: number | null;
  scale: Vec2 | null;
}

export class TextureInfoUtils {
  static applyJSON(info: TextureInfo, json: Partial<TextureInfoJSON>) {
    if (json.texCoord) info.setTexCoord(json.texCoord);
    if (json.magFilter) info.setMagFilter(json.magFilter);
    if (json.minFilter) info.setMinFilter(json.minFilter);
    if (json.wrapS) info.setWrapS(json.wrapS);
    if (json.wrapT) info.setWrapT(json.wrapT);
  }

  static toJSON(info: TextureInfo): TextureInfoJSON {
    const json: TextureInfoJSON = {
      texCoord: info.getTexCoord(),
      magFilter: info.getMagFilter(),
      minFilter: info.getMinFilter(),
      wrapS: info.getWrapS(),
      wrapT: info.getWrapT(),
      offset: null,
      rotation: null,
      scale: null,
    };

    // Texture transform
    const transform = info?.getExtension<Transform>("KHR_texture_transform");
    if (transform) {
      json.offset = transform.getOffset();
      json.rotation = transform.getRotation();
      json.scale = transform.getScale();
    }

    return json;
  }
}
