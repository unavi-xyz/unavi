import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";
import { YMapEvent } from "yjs";
import { MultiplayerContext } from "3d";

import { LocationObject } from "../../contexts/MultiplayerContext";
import Body from "./Body";

interface Props {
  id: string;
}

export default function OtherPlayer({ id }: Props) {
  const { ydoc } = useContext(MultiplayerContext);

  const position = useRef(new Vector3());
  const rotation = useRef(0);

  useEffect(() => {
    if (!ydoc) return;

    const map = ydoc.getMap("locations");

    function onObserve(e: YMapEvent<any>) {
      if (!e.keysChanged.has(id)) return;

      const object = map.get(id) as LocationObject | undefined;
      if (!object) return;

      position.current.fromArray(object.position);
      rotation.current = object.rotation;
    }

    map.observe(onObserve);

    return () => {
      map.unobserve(onObserve);
    };
  }, [id, ydoc]);

  return <Body position={position} rotation={rotation} />;
}
