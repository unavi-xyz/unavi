import { useEffect, useRef } from "react";
import { MeshStandardMaterial, Texture } from "three";

export interface MaterialProps {
  color?: string;
  emissive?: string;
  metalness?: number;
  roughness?: number;
  opacity?: number;
  flatShading?: boolean;
  map?: Texture;
}

export function Material({
  color = "#ffffff",
  emissive = "#000000",
  metalness = 0,
  roughness = 0.5,
  opacity = 1,
  flatShading = false,
  map,
}: MaterialProps) {
  const ref = useRef<MeshStandardMaterial>(null);

  const transparent = opacity < 1;

  useEffect(() => {
    if (ref.current) ref.current.needsUpdate = true;
  }, [map, transparent, flatShading]);

  return (
    <meshStandardMaterial
      ref={ref}
      color={color}
      emissive={emissive}
      roughness={roughness}
      metalness={metalness}
      opacity={opacity}
      transparent={transparent}
      flatShading={flatShading}
      map={map}
    />
  );
}
