import { SphereArgs, Triplet, useSphere } from "@react-three/cannon";

import { Texture } from "../types";
import Material from "../Material";

export const sphereDefaultParams = {
  position: [0, 0, 0] as Triplet,
  rotation: [0, 0, 0] as Triplet,
  radius: 0.5,
};

interface Props {
  params: typeof sphereDefaultParams;
  textures: { [key: string]: Texture };
}

export function Sphere({ params, textures }: Props) {
  const args: SphereArgs = [params.radius];

  const [ref] = useSphere(() => ({
    args,
    position: params.position,
    rotation: params.rotation,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <sphereGeometry args={args} />
      <Material params={params} textures={textures} />
    </mesh>
  );
}
