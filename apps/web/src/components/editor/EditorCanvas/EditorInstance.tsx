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
  const params = instance.params;

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
      ref={ref}
      onClick={handleClick}
      position={params?.position && new Vector3().fromArray(params.position)}
      rotation={params?.rotation && new Euler().fromArray(params.rotation)}
      scale={params?.scale && new Vector3().fromArray(params.scale)}
    >
      <InstancedAsset instance={instance} />
    </group>
  );
}
