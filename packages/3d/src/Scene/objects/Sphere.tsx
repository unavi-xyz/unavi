import { SphereArgs, useSphere } from "@react-three/cannon";

import { CoreProperties, Properties } from "../types";
import { MeshMaterial } from "../modules/MeshMaterial";

export type ISphere = CoreProperties & Pick<Properties, "radius" | "material">;

export const sphereDefaultProperties: ISphere = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  radius: 0.5,
  material: undefined,
};

export function Sphere({ properties = sphereDefaultProperties }) {
  const args: SphereArgs = [properties.radius];

  const [ref] = useSphere(() => ({
    args,
    position: properties.position,
    rotation: properties.rotation,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <sphereGeometry args={args} />
      <MeshMaterial material={properties.material} />
    </mesh>
  );
}
