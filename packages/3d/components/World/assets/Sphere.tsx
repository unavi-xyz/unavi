import { Triplet } from "@react-three/cannon";
import { Params } from "../types";

export const sphereDefaultParams = {
  position: [0, 0, 0] as Triplet,
  rotation: [0, 0, 0] as Triplet,
  scale: [1, 1, 1] as Triplet,
};

interface Props {
  params: Partial<Params>;
}

export function Sphere({ params }: Props) {
  return (
    <mesh>
      <sphereGeometry args={[0.5, 16, 16]} />
      <meshStandardMaterial />
    </mesh>
  );
}
