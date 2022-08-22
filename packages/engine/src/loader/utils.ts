import { TextureInfo } from "@gltf-transform/core";
import { ComponentType } from "bitecs";

import { TextureInfo as TextureInfoType } from "../ecs/components";

export function loadTextureInfo(
  info: TextureInfo,
  component: ComponentType<typeof TextureInfoType>,
  eid: number
) {
  const magFilter = info.getMagFilter();
  component.magFilter[eid] = magFilter ?? 9729;

  const minFilter = info.getMinFilter();
  component.minFilter[eid] = minFilter ?? 9987;

  component.texCoord[eid] = info.getTexCoord();
  component.wrapS[eid] = info.getWrapS();
  component.wrapT[eid] = info.getWrapT();
}
