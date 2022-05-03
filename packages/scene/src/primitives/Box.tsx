import { Triplet, useBox } from "@react-three/cannon";

export type IBox = {};

export const boxDefaultParams: IBox = {};

interface Props {
  params?: IBox;
}

export default function Box({ params = boxDefaultParams }: Props) {
  const args: Triplet = [1, 1, 1];

  const [ref] = useBox(() => ({
    args,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
