import React, { useEffect, useRef } from "react";
import { Group, Vector3 } from "three";

import { Entity } from "../types";
import Module from "./Module/Module";

const tempVec3 = new Vector3();

interface Props {
  entity: Entity;
  children: React.ReactNode;
}

export const EntityComponent = React.forwardRef<Group, Props>(
  ({ entity, children }, ref) => {
    const localRef = useRef<Group | null>(null);

    //set the transform whenever it changes
    useEffect(() => {
      if (!localRef.current) return;

      localRef.current.position.fromArray(entity.transform.position);
      localRef.current.scale.fromArray(entity.transform.scale);
      localRef.current.rotation.setFromVector3(
        tempVec3.fromArray(entity.transform.rotation)
      );
    }, [entity]);

    function handleRef(node: Group | null) {
      localRef.current = node;

      if (typeof ref === "function") ref(localRef.current);
      else if (ref) ref.current = node;
    }

    return (
      <group ref={handleRef as any}>
        <group>
          {entity.modules.map((module) => (
            <Module
              key={module.id}
              module={module}
              entity={entity}
              entityRef={localRef}
            />
          ))}
        </group>

        {children}
      </group>
    );
  }
);
