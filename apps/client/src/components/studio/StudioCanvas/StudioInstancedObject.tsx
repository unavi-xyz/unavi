import { useEffect, useRef } from "react";
import { Group } from "three";

import { ObjectComponent, TreeObject } from "scene";

import { useStudioStore } from "../../../helpers/studio/store";

interface Props {
  object: TreeObject;
}

export default function StudioInstancedObject({ object }: Props) {
  const ref = useRef<Group>(null);

  const setRef = useStudioStore((state) => state.setRef);
  const removeRef = useStudioStore((state) => state.removeRef);

  useEffect(() => {
    setRef(object.id, ref);
    return () => removeRef(object.id);
  }, [object, removeRef, setRef]);

  return (
    <group
      onClick={(e) => {
        e.stopPropagation();
        useStudioStore.setState({ selectedId: object.id });
      }}
    >
      {object && (
        <ObjectComponent ref={ref} object={object}>
          {object.children.map((child) => (
            <StudioInstancedObject key={child.id} object={child} />
          ))}
        </ObjectComponent>
      )}
    </group>
  );
}
