import { useContext, useEffect, useRef } from "react";
import { Triplet } from "@react-three/cannon";
import { Vector3 } from "three";
import { YMapEvent } from "yjs";
import { MultiplayerContext } from "3d";

import Body from "./Body";

interface Props {
  id: string;
}

export default function OtherPlayer({ id }: Props) {
  const { ydoc } = useContext(MultiplayerContext);

  const position = useRef(new Vector3());

  useEffect(() => {
    if (!ydoc) return;

    const map = ydoc.getMap("positions");

    function onObserve(e: YMapEvent<any>) {
      if (!e.keysChanged.has(id)) return;

      const pos = map.get(id) as Triplet;
      if (!pos) return;

      position.current.fromArray(pos);
    }

    map.observe(onObserve);

    return () => {
      map.unobserve(onObserve);
    };
  }, [id, ydoc]);

  return (
    <group>
      <Body position={position} />
    </group>
  );
}
