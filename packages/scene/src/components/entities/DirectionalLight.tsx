export interface DirectionalLightProps {
  color?: string;
  intensity?: number;
}

export function DirectionalLight({
  color = "#ffffff",
  intensity = 1,
}: DirectionalLightProps) {
  return <directionalLight args={[color, intensity]} />;
}
