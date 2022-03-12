import { useRef, useState } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { Group } from "three";
import { useAtom } from "jotai";
import { Instance, InstancedAsset } from "3d";

import { selectedAtom, usingGizmoAtom } from "../../../helpers/editor/state";

interface Props {
  instance: Instance;
}

export default function EditorInstance({ instance }: Props) {
  const params = instance.params;

  const ref = useRef<Group>();

  const [, setSelected] = useAtom(selectedAtom);
  const [usingGizmo] = useAtom(usingGizmoAtom);

  const [original, setOriginal] = useState(JSON.parse(JSON.stringify(params)));
  const [key, setKey] = useState(0);

  function handleClick(e: ThreeEvent<MouseEvent>) {
    if (usingGizmo) return;
    e.stopPropagation();
    setSelected({ instance, ref });
  }

  //you cant change a cannon object's args after being created,
  //so use this monstrosity to force the component to re-mount if they change
  if (JSON.stringify(params) !== JSON.stringify(original)) {
    setKey(Math.random());
    setOriginal(JSON.parse(JSON.stringify(params)));
  }

  //set the position in this parent component, rather than the asset
  //this is so we can use TransformControls on this group
  //its kinda cringe 😣, but allows us to keep editor code out of each Asset
  const usedInstance: Instance = JSON.parse(JSON.stringify(instance));
  usedInstance.params.position = [0, 0, 0];

  return (
    <group key={key} ref={ref} onClick={handleClick} position={params.position}>
      <InstancedAsset instance={usedInstance} />
    </group>
  );
}
