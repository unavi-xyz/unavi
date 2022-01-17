import { useEffect, useRef } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { getObjectComponent, SceneObject } from "3d";

import { useStore } from "../state/useStore";

interface Props {
  object: SceneObject;
}

export default function EditorObject({ object }: Props) {
  const ref = useRef();

  const setSelected = useStore((state) => state.setSelected);

  function handleClick(e: ThreeEvent<MouseEvent>) {
    e.stopPropagation();
    setSelected(object, ref);
  }

  useEffect(() => {
    object.ref = ref;
    object.load();
  }, [object, ref]);

  return (
    <group ref={ref} onClick={handleClick}>
      {getObjectComponent(object)}
    </group>
  );
}
