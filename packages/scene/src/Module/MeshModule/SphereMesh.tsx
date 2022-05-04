export type ISphere = {};

export const sphereDefaultParams: ISphere = {};

interface Props {
  params?: ISphere;
}

export default function SphereMesh({ params = sphereDefaultParams }: Props) {
  return (
    <mesh>
      <sphereBufferGeometry />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
