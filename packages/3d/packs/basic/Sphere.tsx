import { useSphere } from "@react-three/cannon";

import { PHYSICS_GROUPS, SceneObject } from "../..";

interface Props {
  object: SceneObject;
  editor?: boolean;
}

export function Sphere({ object, editor = false }: Props) {
  const { params } = object;

  const args: [number] = editor ? [0.5] : [params.radius];

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
