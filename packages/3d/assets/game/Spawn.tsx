import { Triplet } from "@react-three/cannon";

import { PARAMS, PARAM_NAMES } from "../..";

type SphereParams = Pick<PARAMS, PARAM_NAMES.position>;

export const SpawnDefault: SphereParams = {
  position: [0, 0, 0],
};

const HEIGHT = 1.6;

export function Spawn() {
  const args: Triplet = [0.8, HEIGHT, 0.8];

  return (
    <mesh position={[0, HEIGHT / 2, 0]}>
      <boxGeometry args={args} />
      <meshPhongMaterial color="#ffaaaa" opacity={0.8} transparent={true} />
    </mesh>
  );
}
