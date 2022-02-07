import { Triplet, useBox } from "@react-three/cannon";

import { PHYSICS_GROUPS } from "../..";
import { SceneObject } from "..";

interface Props {
  object: SceneObject;
  editor?: boolean;
}

export function Box({ object, editor = false }: Props) {
  const args: Triplet = editor ? [1, 1, 1] : object.scale;

  const disablePhysics = object.physEnabled === false || editor;

  const [ref] = useBox(() => ({
    args,
    position: editor ? undefined : object.position,
    rotation: editor ? undefined : object.rotation,
    mass: object.mass,
    type: editor ? "Static" : object.physType,
    collisionFilterMask: disablePhysics ? PHYSICS_GROUPS.NONE : undefined,
  }));

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <meshPhongMaterial
        color={object.color}
        opacity={object.opacity}
        transparent={Boolean(object.opacity < 1)}
      />
    </mesh>
  );
}
