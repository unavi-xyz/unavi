export interface AmbientLightProps {
  color?: string;
  intensity?: number;
}

export function AmbientLight({
  color = "#ffffff",
  intensity = 1,
}: AmbientLightProps) {
  return <ambientLight args={[color, intensity]} />;
}
