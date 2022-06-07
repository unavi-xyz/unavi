export interface PointLightProps {
  color?: string;
  intensity?: number;
  distance?: number;
  decay?: number;
}

export function PointLight({
  color = "#ffffff",
  intensity = 1,
  distance = 0,
  decay = 1,
}: PointLightProps) {
  return (
    <pointLight
      castShadow
      args={[color, intensity, distance, decay]}
      shadow-mapSize-width={1024}
      shadow-mapSize-height={1024}
    />
  );
}
