export interface DirectionalLightProps {
  color?: string;
  intensity?: number;
}

export function DirectionalLight({ color = "#ffffff", intensity = 1 }: DirectionalLightProps) {
  return (
    <directionalLight
      castShadow
      args={[color, intensity]}
      shadow-mapSize-width={8192}
      shadow-mapSize-height={8192}
      shadow-camera-far={100}
      shadow-camera-left={-30}
      shadow-camera-right={30}
      shadow-camera-top={30}
      shadow-camera-bottom={-30}
    />
  );
}
