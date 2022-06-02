export interface AmbientProps {
  color?: string;
  intensity?: number;
  distance?: number;
  decay?: number;
}

export function AmbientLight({
  color = "#ffffff",
  intensity = 1,
}: AmbientProps) {
  return <ambientLight args={[color, intensity]} />;
}
