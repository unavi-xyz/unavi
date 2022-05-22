import { ThreeEvent } from "@react-three/fiber";
import { Ref, useEffect, useRef } from "react";
import { Group } from "three";

import { Entity, EntityComponent } from "@wired-xr/scene";

import { useStudioStore } from "../../../helpers/studio/store";

interface Props {
  entity: Entity;
}

export default function StudioInstancedEntity({ entity }: Props) {
  const ref = useRef<Group>(null);

  const setRef = useStudioStore((state) => state.setRef);
  const removeRef = useStudioStore((state) => state.removeRef);

  useEffect(() => {
    setRef(entity.id, ref);
    return () => removeRef(entity.id);
  }, [entity, removeRef, setRef]);

  function handlePointerUp(e: ThreeEvent<PointerEvent>) {
    // @ts-ignore
    if (e.button !== 0) return;
    if (useStudioStore.getState().usingGizmo) return;
    e.stopPropagation();
    useStudioStore.setState({ selectedId: entity.id });
  }

  return (
    <group onPointerUp={handlePointerUp}>
      {entity && (
        <EntityComponent ref={ref as any} entity={entity}>
          {entity.children.map((child) => (
            <StudioInstancedEntity key={child.id} entity={child} />
          ))}
        </EntityComponent>
      )}
    </group>
  );
}
