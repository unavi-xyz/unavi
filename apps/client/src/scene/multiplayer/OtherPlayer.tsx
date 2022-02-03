import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";
import { YMapEvent } from "yjs";
import { MultiplayerContext } from "3d";

import Body from "./Body";

interface Props {}

export default function OtherPlayer({}) {
  const { ymap } = useContext(MultiplayerContext);

  const position = useRef(new Vector3());

  // useEffect(() => {
  // if (!ymap) return;
  // function onObserve(e: YMapEvent<any>) {
  //   if (!e.keysChanged.has(member.userId)) return;
  //   const pos = ymap.get(member.userId);
  //   position.current.fromArray(pos);
  // }
  // ymap.observe(onObserve);
  // return () => {
  //   ymap.unobserve(onObserve);
  // };
  // }, [member.userId, ymap]);

  return (
    <group>
      <Body position={position} />
    </group>
  );
}
