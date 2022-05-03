import React, { useEffect, useRef } from "react";
import { Group, Vector3 } from "three";

import { PRIMITIVES } from "../primitives";
import { TreeObject } from "../types";

const tempVec3 = new Vector3();

interface Props {
  object: TreeObject;
  children: React.ReactNode;
}

export const ObjectComponent = React.forwardRef<Group, Props>(
  ({ object, children }, ref) => {
    const localRef = useRef<Group | null>(null);

    useEffect(() => {
      if (!localRef.current) return;

      localRef.current.position.fromArray(object.position);
      localRef.current.scale.fromArray(object.scale);
      localRef.current.rotation.setFromVector3(
        tempVec3.fromArray(object.rotation)
      );
    }, [object]);

    function getRef(node: Group | null) {
      localRef.current = node;

      if (typeof ref === "function") ref(localRef.current);
      else if (ref) ref.current = node;
    }

    if (object.type === "Primitive") {
      const Component = PRIMITIVES[object.primitive]["Component"];

      return (
        <group ref={getRef}>
          <Component params={object.params as any} />
          <group>{children}</group>
        </group>
      );
    }

    return <group ref={getRef}>{children}</group>;
  }
);
