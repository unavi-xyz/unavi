import { Triplet } from "@react-three/cannon";

export function Box() {
  const args: Triplet = [1, 1, 1];

  return (
    <mesh>
      <boxGeometry args={args} />
      <meshPhongMaterial />
    </mesh>
  );
}
