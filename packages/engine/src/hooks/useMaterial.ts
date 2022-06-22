import { TextureLoader } from "three";

import { MaterialProps } from "../components/entities/Material";
import { useAsset } from "./useAsset";
import { useImage } from "./useImage";

export function useMaterial(id: string | undefined): MaterialProps | undefined {
  const asset = useAsset<"material">(id);
  const material = asset?.data;
  const textureImage = useImage(material?.textureId);

  if (material && textureImage) {
    const textureLoader = new TextureLoader();
    const map = textureLoader.load(textureImage);
    return { ...material, map };
  }

  return material;
}
