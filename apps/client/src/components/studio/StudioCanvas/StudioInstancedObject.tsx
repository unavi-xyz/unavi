import produce from "immer";
import { useEffect, useRef, useState } from "react";
import { ObjectComponent, TreeObject } from "scene";
import { Group } from "three";

import { useStudioStore } from "../../../helpers/studio/store";

interface Props {
  object: TreeObject;
}

export default function StudioInstancedObject({ object }: Props) {
  const ref = useRef<Group>();

  const setRef = useStudioStore((state) => state.setRef);
  const removeRef = useStudioStore((state) => state.removeRef);

  const [usedObject, setUsedObject] = useState<typeof object>();

  useEffect(() => {
    setRef(object.id, ref);
    return () => removeRef(object.id);
  }, [object, removeRef, setRef]);

  //strip position and rotation from the object
  //we add them to the group at this level
  //so that we can use the group as a reference for Gizmo
  //? could this be moved into ObjectComponent?
  useEffect(() => {
    if (!ref.current || !object?.params) {
      setUsedObject(object);
      ref.current?.position.set(0, 0, 0);
      ref.current?.rotation.set(0, 0, 0);
      return;
    }

    ref.current.position.set(...object.params.position);
    ref.current.rotation.set(...object.params.rotation);

    const newObject = produce(object, (draft) => {
      if (draft.params) {
        draft.params.position = [0, 0, 0];
        draft.params.rotation = [0, 0, 0];
      }
    });

    setUsedObject(newObject);
  }, [object]);

  return (
    <group
      ref={ref}
      onClick={(e) => {
        e.stopPropagation();
        useStudioStore.setState({ selected: object });
      }}
    >
      {usedObject && (
        <ObjectComponent object={usedObject}>
          {usedObject.children.map((child) => (
            <StudioInstancedObject key={child.id} object={child} />
          ))}
        </ObjectComponent>
      )}
    </group>
  );
}
