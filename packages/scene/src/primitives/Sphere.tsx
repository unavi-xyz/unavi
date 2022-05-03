import { Triplet, useSphere } from "@react-three/cannon";

export type ISphere = {
  radius: number;
};

export const sphereDefaultParams: ISphere = {
  radius: 0.5,
};

interface Props {
  params?: ISphere;
}

export default function Sphere({ params = sphereDefaultParams }: Props) {
  const args: [number] = [params.radius];

  const [ref] = useSphere(() => ({
    args,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <sphereBufferGeometry args={args} />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
