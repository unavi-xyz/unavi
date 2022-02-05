import { useSphere } from "@react-three/cannon";

import { PHYSICS_GROUPS } from "../..";
import { SceneObject } from "..";

interface Props {
  object: SceneObject;
  editor?: boolean;
}

export function Sphere({ object, editor = false }: Props) {
  const args: [number] = editor ? [0.5] : [object.radius];

  const disablePhysics = object.physEnabled === false || editor;

  const [ref] = useSphere(() => ({
    args,
    position: editor ? undefined : object.position,
    rotation: editor ? undefined : object.rotation,
    mass: object.mass,
    type: editor ? "Static" : object.physType,
    collisionFilterMask: disablePhysics ? PHYSICS_GROUPS.NONE : undefined,
  }));

  return (
    <mesh ref={ref}>
      <sphereBufferGeometry args={args} />
      <meshPhongMaterial
        color={object.color}
        opacity={object.opacity}
        transparent={Boolean(object.opacity < 1)}
      />
    </mesh>
  );
}
