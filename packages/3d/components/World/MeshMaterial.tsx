import { useEffect, useRef, useState } from "react";
import {
  MeshStandardMaterial,
  Texture as ThreeTexture,
  TextureLoader,
} from "three";

import { Material, Texture } from "./types";

const loader = new TextureLoader();

export const defaultMaterial: Material = {
  color: "#ffffff",
  opacity: 1,
  texture: undefined,
};

interface Props {
  material: Material | undefined;
  textures: { [key: string]: Texture };
}

export function MeshMaterial({ material, textures }: Props) {
  const ref = useRef<MeshStandardMaterial>();

  const [texture, setTexture] = useState<ThreeTexture>();

  useEffect(() => {
    if (!material?.texture || !textures) {
      setTexture(undefined);
      return;
    }

    const texture = textures[material.texture];
    if (!texture) return;

    loader.loadAsync(texture.value).then((res) => {
      setTexture(res);
    });
  }, [material, textures]);

  useEffect(() => {
    if (ref.current) ref.current.needsUpdate = true;
  }, [texture]);

  if (!material) return null;

  return (
    <meshStandardMaterial
      ref={ref}
      map={texture}
      color={material.color}
      opacity={material.opacity}
      transparent={material.opacity < 1}
    />
  );
}
