import { GLTF, TextureInfo } from "@gltf-transform/core";

export interface TextureInfoJSON {
  texCoord: number;
  magFilter: GLTF.TextureMagFilter | null;
  minFilter: GLTF.TextureMinFilter | null;
  wrapS: GLTF.TextureWrapMode;
  wrapT: GLTF.TextureWrapMode;
}

export class TextureInfoUtils {
  applyJSON(info: TextureInfo, json: Partial<TextureInfoJSON>) {
    if (json.texCoord) info.setTexCoord(json.texCoord);
    if (json.magFilter) info.setMagFilter(json.magFilter);
    if (json.minFilter) info.setMinFilter(json.minFilter);
    if (json.wrapS) info.setWrapS(json.wrapS);
    if (json.wrapT) info.setWrapT(json.wrapT);
  }

  toJSON(info: TextureInfo): TextureInfoJSON {
    return {
      texCoord: info.getTexCoord(),
      magFilter: info.getMagFilter(),
      minFilter: info.getMinFilter(),
      wrapS: info.getWrapS(),
      wrapT: info.getWrapT(),
    };
  }
}
