import { SphereArgs, Triplet, useSphere } from "@react-three/cannon";

import { Texture } from "../types";
import { MeshMaterial, defaultMaterial } from "../MeshMaterial";

export const sphereDefaultParams = {
  position: [0, 0, 0] as Triplet,
  rotation: [0, 0, 0] as Triplet,
  radius: 0.5,
  material: defaultMaterial,
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
      <MeshMaterial material={params.material} textures={textures} />
    </mesh>
  );
}
