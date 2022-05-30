import React, { useEffect, useRef } from "react";
import { Group, Vector3 } from "three";

import { Entity } from "../types";
import Module from "./Module/Module";

const tempVec3 = new Vector3();

interface Props {
  groupRef?: React.RefObject<Group>;
  entity: Entity;
  children: React.ReactNode;
}

export function EntityComponent({
  groupRef = React.createRef(),
  entity,
  children,
}: Props) {
  //set the transform whenever it changes
  useEffect(() => {
    if (!groupRef.current) return;

    groupRef.current.position.fromArray(entity.transform.position);
    groupRef.current.scale.fromArray(entity.transform.scale);
    groupRef.current.rotation.setFromVector3(
      tempVec3.fromArray(entity.transform.rotation)
    );
  }, [entity]);

  return (
    <group ref={groupRef as any}>
      <group>
        {entity.modules.map((module) => (
          <Module
            key={module.id}
            module={module}
            entity={entity}
            entityRef={groupRef}
          />
        ))}
      </group>

      {children}
    </group>
  );
}
