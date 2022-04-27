import { Triplet, useBox } from "@react-three/cannon";
import { useTexture } from "@react-three/drei";
import { MirroredRepeatWrapping } from "three";

const THICKNESS = 0.1;

interface Props {
  size?: number;
}

export function Ground({ size = 10 }: Props) {
  const texture = useTexture("/images/grid.png");

  const args: Triplet = [size, THICKNESS, size];

  const [ref] = useBox(() => ({
    args,
    type: "Static",
    position: [0, -THICKNESS / 2, 0],
  }));

  texture.wrapS = MirroredRepeatWrapping;
  texture.wrapT = MirroredRepeatWrapping;
  texture.repeat.set(size / 2, size / 2);

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <meshPhongMaterial map={texture} />
    </mesh>
  );
}
