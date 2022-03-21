import { useEffect, useRef, useState } from "react";
import {
  MeshPhysicalMaterial,
  MeshToonMaterial,
  Texture as ThreeTexture,
  TextureLoader,
} from "three";

import { Material, Texture } from "./types";

const loader = new TextureLoader();

export const defaultMaterial: Material = {
  type: "physical",
  color: "#ffffff",
  emissive: "#000000",
  sheenColor: "#000000",
  opacity: 1,
  metalness: 0,
  roughness: 0.5,
  reflectivity: 0.5,
  clearcoat: 0,
  sheen: 0,
  texture: undefined,
  flatShading: false,
};

interface Props {
  material: Material | undefined;
  textures: { [key: string]: Texture };
}

export function MeshMaterial({ material, textures }: Props) {
  const physicalRef = useRef<MeshPhysicalMaterial>();
  const toonRef = useRef<MeshToonMaterial>();

  const [texture, setTexture] = useState<ThreeTexture>();

  const isToon = material?.type === "toon";

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
    if (isToon) {
      if (toonRef.current) toonRef.current.needsUpdate = true;
    } else {
      if (physicalRef.current) physicalRef.current.needsUpdate = true;
    }
  }, [texture]);

  useEffect(() => {
    if (physicalRef.current) physicalRef.current.needsUpdate = true;
  }, [material?.flatShading]);

  if (!material) return null;

  if (isToon) {
    return (
      <meshToonMaterial
        ref={toonRef}
        map={texture}
        color={material.color}
        emissive={material.emissive}
        opacity={material.opacity}
        transparent={material.opacity < 1}
      />
    );
  }

  return (
    <meshPhysicalMaterial
      ref={physicalRef}
      map={texture}
      color={material.color}
      emissive={material.emissive}
      opacity={material.opacity}
      transmission={Math.abs(1 - material.opacity)}
      reflectivity={material.reflectivity}
      metalness={material.metalness}
      roughness={material.roughness}
      clearcoat={material.clearcoat}
      sheen={material.sheen}
      sheenColor={material.sheenColor}
      transparent={material.opacity < 1}
      flatShading={material.flatShading}
    />
  );
}
