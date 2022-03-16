import { Triplet, useBox } from "@react-three/cannon";
import { Material } from "../Material";

export const boxDefaultParams = {
  position: [0, 0, 0] as Triplet,
  rotation: [0, 0, 0] as Triplet,
  scale: [1, 1, 1] as Triplet,
};

interface Props {
  params: typeof boxDefaultParams;
}

export function Box({ params }: Props) {
  const args: Triplet = params.scale;

  const [ref] = useBox(() => ({
    args,
    position: params.position,
    rotation: params.rotation,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <Material params={params} />
    </mesh>
  );
}
