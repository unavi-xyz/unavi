import { useRef } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { SceneObject } from "3d";

import { useStore } from "../state";

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

  return (
    <group ref={ref} onClick={handleClick}>
      {object.component}
    </group>
  );
}
