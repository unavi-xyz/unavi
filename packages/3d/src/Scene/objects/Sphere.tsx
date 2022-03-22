import { SphereArgs, useSphere } from "@react-three/cannon";

import { Properties } from "../types";
import { defaultMaterial, MeshMaterial } from "../modules/MeshMaterial";

export type ISphere = Pick<
  Properties,
  "position" | "rotation" | "radius" | "material"
>;

export const sphereDefaultProperties: ISphere = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  radius: 0.5,
  material: defaultMaterial,
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
