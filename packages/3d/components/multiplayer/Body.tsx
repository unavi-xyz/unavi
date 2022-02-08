import { MutableRefObject, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Group, Vector3 } from "three";
import { Avatar } from "avatars";

import { PUBLISH_INTERVAL } from "../../constants";

interface Props {
  position: MutableRefObject<Vector3>;
  rotation: MutableRefObject<Vector3>;
}

export default function Body({ position, rotation }: Props) {
  const group = useRef<Group>();

  const deltaPos = useRef(0);
  const lastPos = useRef(new Vector3());
  const currentPos = useRef(new Vector3());
  const interpPos = useRef(new Vector3());

  useFrame((_, delta) => {
    //interpolation
    deltaPos.current += delta;

    if (!currentPos.current.equals(position.current)) {
      lastPos.current.copy(currentPos.current);
      currentPos.current.copy(position.current);
      deltaPos.current = 0;
    }

    const alpha = Math.min(deltaPos.current * (1000 / PUBLISH_INTERVAL), 1);
    interpPos.current.lerpVectors(lastPos.current, currentPos.current, alpha);

    group.current?.position.copy(interpPos.current);
    group.current?.rotation.setFromVector3(rotation.current);
  });

  return (
    <group ref={group}>
      <Avatar />
    </group>
  );
}
