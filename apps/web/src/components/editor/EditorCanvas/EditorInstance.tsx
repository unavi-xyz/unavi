import { useRef } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { Euler, Group, Vector3 } from "three";
import { useAtom } from "jotai";
import { Instance, InstancedAsset } from "3d";

import { selectedAtom, usingGizmoAtom } from "../../../helpers/editor/state";

interface Props {
  instance: Instance;
}

export default function EditorInstance({ instance }: Props) {
  const ref = useRef<Group>();

  const [, setSelected] = useAtom(selectedAtom);

  const [usingGizmo] = useAtom(usingGizmoAtom);

  function handleClick(e: ThreeEvent<MouseEvent>) {
    if (usingGizmo) return;
    e.stopPropagation();
    setSelected({ instance, ref });
  }

  return (
    <group
      position={new Vector3().fromArray(instance.params.position)}
      rotation={new Euler().fromArray(instance.params.rotation)}
      scale={new Vector3().fromArray(instance.params.scale)}
      ref={ref}
      onClick={handleClick}
    >
      <InstancedAsset instance={instance} />
    </group>
  );
}
