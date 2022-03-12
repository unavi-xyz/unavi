import { Euler } from "three";
import { useVRM } from "./useVRM";

interface Props {
  src: string;
}

export function Avatar({ src }: Props) {
  const vrm = useVRM(src);

  if (!vrm?.scene) return null;

  return (
    <group rotation={new Euler(0, Math.PI, 0)}>
      <primitive object={vrm?.scene} />
    </group>
  );
}
