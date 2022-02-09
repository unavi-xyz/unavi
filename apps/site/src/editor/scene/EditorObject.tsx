import { useEffect, useRef } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { Group } from "three";
import { InstancedObject } from "3d";

import { useStore } from "../state/useStore";
import { useScene } from "../state/useScene";

interface Props {
  id: string;
}

export default function EditorObject({ id }: Props) {
  const setSelected = useStore((state) => state.setSelected);
  const object = useScene((state) => state.scene[id]);

  const ref = useRef<Group>();

  useEffect(() => {
    object.ref = ref;
    object.load();
  }, [object]);

  function handleClick(e: ThreeEvent<MouseEvent>) {
    e.stopPropagation();
    setSelected(object);
  }

  return (
    <group ref={ref} onClick={handleClick}>
      <InstancedObject instance={object.instance} editor />
    </group>
  );
}
