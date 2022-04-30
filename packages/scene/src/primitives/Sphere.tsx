import { Triplet, useSphere } from "@react-three/cannon";

export type ISphere = {
  position: Triplet;
  rotation: Triplet;
  radius: number;
};

export const sphereDefaultParams: ISphere = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  radius: 0.5,
};

interface Props {
  params?: ISphere;
}

export default function Sphere({ params = sphereDefaultParams }: Props) {
  const args: [number] = [params.radius];

  const [ref] = useSphere(() => ({
    args,
    position: params.position,
    rotation: params.rotation,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <sphereBufferGeometry args={args} />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
