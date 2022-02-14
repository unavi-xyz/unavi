import { Triplet, useBox } from "@react-three/cannon";

import { PARAMS, PARAM_NAMES, PHYSICS_GROUPS } from "../..";

type BoxParams = Pick<
  PARAMS,
  | PARAM_NAMES.position
  | PARAM_NAMES.rotation
  | PARAM_NAMES.scale
  | PARAM_NAMES.color
  | PARAM_NAMES.opacity
  | PARAM_NAMES.physEnabled
  | PARAM_NAMES.physType
  | PARAM_NAMES.mass
>;

export const BoxDefault: BoxParams = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
  color: "#ffffff",
  opacity: 1,
  physEnabled: true,
  physType: "Static",
  mass: 1,
};

interface Props {
  params: BoxParams;
  editor?: boolean;
}

export function Box({ params, editor = false }: Props) {
  const args: Triplet = editor ? [1, 1, 1] : params.scale;

  const disablePhysics = params.physEnabled === false || editor;

  const [ref] = useBox(() => ({
    args,
    position: editor ? undefined : params.position,
    rotation: editor ? undefined : params.rotation,
    mass: params.mass,
    type: editor ? "Static" : params.physType,
    collisionFilterMask: disablePhysics ? PHYSICS_GROUPS.NONE : undefined,
  }));

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <meshPhongMaterial
        color={params.color}
        opacity={params.opacity}
        transparent={Boolean(params.opacity < 1)}
      />
    </mesh>
  );
}
