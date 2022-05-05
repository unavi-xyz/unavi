export type IBox = {};

export const boxDefaultParams: IBox = {};

interface Props {
  params?: IBox;
}

export default function BoxMesh({ params = boxDefaultParams }: Props) {
  return (
    <mesh>
      <boxBufferGeometry />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
