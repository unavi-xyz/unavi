import { useState, useEffect, useMemo, useRef } from "react";
import { ThreeEvent } from "@react-three/fiber";
import { Group } from "three";
import { InstancedObject } from "3d";

import { editorManager, useStore } from "../helpers/store";

interface Props {
  id: string;
}

export default function EditorInstance({ id }: Props) {
  const ref = useRef<Group>();

  const instance = useStore((state) => state.scene.instances[id]);
  const properties = useStore((state) => state.scene.instances[id].properties);

  const [initial] = useState(properties);

  const usedInstance = useMemo(() => {
    const modified: typeof properties = JSON.parse(JSON.stringify(properties));

    modified.position[0] = properties.position[0] - initial.position[0];
    modified.position[1] = properties.position[1] - initial.position[1];
    modified.position[2] = properties.position[2] - initial.position[2];

    modified.rotation[0] = properties.rotation[0] - initial.rotation[0];
    modified.rotation[1] = properties.rotation[1] - initial.rotation[1];
    modified.rotation[2] = properties.rotation[2] - initial.rotation[2];

    if ("scale" in modified) modified.scale = [1, 1, 1];

    const newInstance = { ...instance, properties: modified };
    return newInstance;
  }, [initial, instance, properties]);

  useEffect(() => {
    if (!ref.current) return;
    ref.current.position.set(...properties.position);
    ref.current.rotation.set(...properties.rotation);
    if ("scale" in properties) ref.current.scale.set(...properties.scale);
  }, [properties]);

  function handleClick(e: ThreeEvent<MouseEvent>) {
    if (useStore.getState().usingGizmo) return;
    e.stopPropagation();

    const selected = { id: instance.id, ref };
    editorManager.setSelected(selected);
  }

  return (
    <group
      ref={ref}
      onClick={handleClick}
      position={properties.position}
      rotation={properties.rotation}
      scale={"scale" in properties ? properties.scale : undefined}
    >
      <InstancedObject instance={usedInstance} />
    </group>
  );
}
