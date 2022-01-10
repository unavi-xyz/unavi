import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";
import { MultiplayerContext } from "matrix";
import { RoomMember } from "matrix-js-sdk";
import { YMapEvent } from "yjs";

import Body from "./Body";

interface Props {
  member: RoomMember;
}

export default function OtherPlayer({ member }: Props) {
  const { ymap } = useContext(MultiplayerContext);

  const position = useRef(new Vector3());

  useEffect(() => {
    if (!ymap) return;

    function onObserve(e: YMapEvent<any>) {
      if (!e.keysChanged.has(member.userId)) return;

      const pos = ymap.get(member.userId);
      position.current.fromArray(pos);
    }

    ymap.observe(onObserve);

    return () => {
      ymap.unobserve(onObserve);
    };
  }, [member.userId, ymap]);

  return (
    <group>
      <Body position={position} />
    </group>
  );
}
