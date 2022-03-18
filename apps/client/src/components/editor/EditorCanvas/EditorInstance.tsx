import { useState, useEffect, useMemo, useRef } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { Group } from "three";
import { useAtom } from "jotai";
import { InstancedAsset } from "3d";

import { usingGizmoAtom } from "../helpers/state";
import { useStore } from "../helpers/store";

interface Props {
  id: string;
}

export default function EditorInstance({ id }: Props) {
  const ref = useRef<Group>();

  const textures = useStore((state) => state.scene.textures);
  const models = useStore((state) => state.scene.models);
  const instance = useStore((state) => state.scene.instances[id]);
  const params = useStore((state) => state.scene.instances[id].params);

  const [initialParams] = useState(params);
  const usedParams = useMemo(() => {
    const modifiedParams: typeof params = JSON.parse(JSON.stringify(params));

    modifiedParams.position[0] = params.position[0] - initialParams.position[0];
    modifiedParams.position[1] = params.position[1] - initialParams.position[1];
    modifiedParams.position[2] = params.position[2] - initialParams.position[2];

    modifiedParams.rotation[0] = params.rotation[0] - initialParams.rotation[0];
    modifiedParams.rotation[1] = params.rotation[1] - initialParams.rotation[1];
    modifiedParams.rotation[2] = params.rotation[2] - initialParams.rotation[2];

    if (initialParams?.scale) modifiedParams.scale = [1, 1, 1];

    return modifiedParams;
  }, [initialParams, params]);

  useEffect(() => {
    if (!ref.current) return;
    ref.current.position.set(...params.position);
    ref.current.rotation.set(...params.rotation);
    if (params?.scale) ref.current.scale.set(...params.scale);
  }, [params]);

  const [usingGizmo] = useAtom(usingGizmoAtom);

  function handleClick(e: ThreeEvent<MouseEvent>) {
    if (usingGizmo) return;
    e.stopPropagation();
    useStore.getState().setSelected({ id: instance.id, ref });
  }

  return (
    <group
      ref={ref}
      onClick={handleClick}
      position={params.position}
      rotation={params.rotation}
      scale={params?.scale}
    >
      <InstancedAsset
        name={instance.name}
        params={usedParams}
        textures={textures}
        models={models}
      />
    </group>
  );
}
