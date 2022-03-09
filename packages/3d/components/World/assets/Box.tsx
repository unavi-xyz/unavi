import { Params } from "../types";

export const boxDefaultParams = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
};

interface Props {
  params: Partial<Params>;
}

export function Box({ params }: Props) {
  return (
    <mesh>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial />
    </mesh>
  );
}
