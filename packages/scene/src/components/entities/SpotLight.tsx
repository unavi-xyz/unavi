import { useFrame } from "@react-three/fiber";
import { useRef } from "react";
import { SpotLight as ThreeSpotLight, Vector3 } from "three";

export interface SpotLightProps {
  color?: string;
  intensity?: number;
  distance?: number;
  decay?: number;
  angle?: number;
  penumbra?: number;
}

export function SpotLight({
  color = "#ffffff",
  intensity = 1,
  distance = 0,
  decay = 1,
  angle = Math.PI / 3,
  penumbra = 0,
}: SpotLightProps) {
  const ref = useRef<ThreeSpotLight>(null);
  const directionRef = useRef(new Vector3());

  useFrame(() => {
    if (!ref.current) return;

    if (!ref.current.target.parent) {
      ref.current.add(ref.current.target);
    }

    //set target to directly in front of the light
    ref.current.getWorldDirection(directionRef.current);
    ref.current.target.position.copy(directionRef.current);
  });

  return (
    <spotLight
      ref={ref}
      castShadow
      args={[color, intensity, distance, angle, penumbra, decay]}
      shadow-mapSize-width={2048}
      shadow-mapSize-height={2048}
    />
  );
}
