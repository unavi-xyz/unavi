import { SphereArgs, useSphere } from "@react-three/cannon";

import { PARAMS, PARAM_NAMES, PHYSICS_GROUPS } from "../..";

type SphereParams = Pick<
  PARAMS,
  | PARAM_NAMES.position
  | PARAM_NAMES.rotation
  | PARAM_NAMES.radius
  | PARAM_NAMES.color
  | PARAM_NAMES.opacity
  | PARAM_NAMES.physEnabled
  | PARAM_NAMES.physType
  | PARAM_NAMES.mass
>;

export const SphereDefault: SphereParams = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  radius: 0.5,
  color: "#ffffff",
  opacity: 1,
  physEnabled: true,
  physType: "Static",
  mass: 1,
};

interface Props {
  params: SphereParams;
  editor?: boolean;
}

export function Sphere({ params, editor = false }: Props) {
  const args: SphereArgs = [params.radius];

  const disablePhysics = params.physEnabled === false || editor;

  const [ref] = useSphere(() => ({
    args,
    position: editor ? undefined : params.position,
    rotation: editor ? undefined : params.rotation,
    mass: params.mass,
    type: editor ? "Static" : params.physType,
    collisionFilterMask: disablePhysics ? PHYSICS_GROUPS.NONE : undefined,
  }));

  return (
    <mesh ref={ref}>
      <sphereBufferGeometry args={args} />
      <meshPhongMaterial
        color={params.color}
        opacity={params.opacity}
        transparent={Boolean(params.opacity < 1)}
      />
    </mesh>
  );
}
