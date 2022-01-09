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

  function onObserve(e: YMapEvent<any>) {
    if (!e.keysChanged.has(member.userId)) return;

    const pos = ymap.get(member.userId);
    position.current.fromArray(pos);
  }

  useEffect(() => {
    if (!ymap) return;
    ymap.observe(onObserve);

    return () => {
      ymap.unobserve(onObserve);
    };
  }, [ymap]);

  return (
    <group>
      <Body position={position} />
    </group>
  );
}
