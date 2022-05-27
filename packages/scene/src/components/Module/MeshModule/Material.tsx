import { useRef } from "react";
import { useEffect } from "react";
import { useState } from "react";
import { MeshStandardMaterial } from "three";
import { Texture } from "three";
import { TextureLoader } from "three";

import { useAsset } from "../../AssetProvider";
import { useMaterial } from "../../MaterialProvider";

interface Props {
  materialId: string | undefined;
}

export default function Material({ materialId }: Props) {
  const ref = useRef<MeshStandardMaterial>();

  const material = useMaterial(materialId);
  const textureAsset = useAsset(material?.textureId);

  const [texture, setTexture] = useState<Texture>();

  const transparent = Boolean((material?.opacity ?? 1) < 1);

  useEffect(() => {
    if (ref.current) ref.current.needsUpdate = true;
  }, [material, texture]);

  useEffect(() => {
    if (textureAsset?.data) {
      const loaded = new TextureLoader().load(textureAsset.data);
      setTexture(loaded);
    }
  }, [textureAsset]);

  return (
    <meshStandardMaterial
      ref={ref as any}
      color={material?.color ?? "#ffffff"}
      emissive={material?.emissive}
      roughness={material?.roughness}
      metalness={material?.metalness}
      opacity={material?.opacity}
      transparent={transparent}
      flatShading={material?.flatShading}
      map={texture as any}
    />
  );
}
