import { Triplet } from "@react-three/cannon";

import { SceneObject } from "../..";

interface Props {
  object: SceneObject;
}

export function Box({ object }: Props) {
  const args: Triplet = [1, 1, 1];

  return (
    <mesh>
      <boxGeometry args={args} />
      <meshPhongMaterial
        color={object.color}
        opacity={object.opacity}
        transparent={Boolean(object.opacity < 1)}
      />
    </mesh>
  );
}
