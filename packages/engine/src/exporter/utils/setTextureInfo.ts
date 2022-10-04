import { TextureInfo } from "@gltf-transform/core";

import { Texture } from "../../scene";

export function setTextureInfo(info: TextureInfo | null, texture: Texture) {
  if (!info) return;

  info.setMagFilter(texture.magFilter);
  info.setMinFilter(texture.minFilter);
  info.setWrapS(texture.wrapS);
  info.setWrapT(texture.wrapT);
}
