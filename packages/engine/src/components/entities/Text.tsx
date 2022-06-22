import { Text as DreiText } from "@react-three/drei";

export interface TextProps {
  text?: string;
  fontSize?: number;
  color?: string;
}

export function Text({
  text = "",
  fontSize = 1,
  color = "#000000",
}: TextProps) {
  return (
    <DreiText color={color} fontSize={fontSize}>
      {text}
    </DreiText>
  );
}
