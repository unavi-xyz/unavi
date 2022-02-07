import { MutableRefObject, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { useSphere } from "@react-three/cannon";
import { Vector3 } from "three";

import { PHYSICS_GROUPS, PUBLISH_INTERVAL } from "../../constants";

const args: [number] = [1];

interface Props {
  position: MutableRefObject<Vector3>;
}

export default function Body({ position }: Props) {
  const deltaPos = useRef(0);
  const lastPos = useRef(new Vector3());
  const currentPos = useRef(new Vector3());
  const interpPos = useRef(new Vector3());

  const [ref, api] = useSphere(() => ({
    args,
    mass: 1,
    type: "Static",
    position: [0, -100, 0],
    collisionFilterMask: PHYSICS_GROUPS.NONE,
  }));

  useFrame((_, delta) => {
    deltaPos.current += delta;

    if (!currentPos.current.equals(position.current)) {
      lastPos.current.copy(currentPos.current);
      currentPos.current.copy(position.current);
      deltaPos.current = 0;
    }

    const alpha = Math.min(deltaPos.current * (1000 / PUBLISH_INTERVAL), 1);
    interpPos.current.lerpVectors(lastPos.current, currentPos.current, alpha);

    api.position.copy(interpPos.current);
  });

  return (
    <mesh ref={ref}>
      <sphereGeometry args={args} />
      <meshPhongMaterial />
    </mesh>
  );
}
