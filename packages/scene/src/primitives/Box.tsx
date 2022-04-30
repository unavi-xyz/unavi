import { Triplet, useBox } from "@react-three/cannon";

export type IBox = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
};

export const boxDefaultParams: IBox = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
};

interface Props {
  params?: IBox;
}

export default function Box({ params = boxDefaultParams }: Props) {
  const args = params.scale;

  const [ref] = useBox(() => ({
    args,
    position: params.position,
    rotation: params.rotation,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
