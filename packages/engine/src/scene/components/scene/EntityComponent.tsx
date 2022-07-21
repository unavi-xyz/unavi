import { ReactNode, RefObject, createRef, useEffect } from "react";
import { Group, Vector3 } from "three";

import { ENTITY_COMPONENTS, IEntity } from "../../types";

const tempVec3 = new Vector3();

interface Props {
  groupRef?: RefObject<Group>;
  entity: IEntity;
  children: ReactNode;
}

export function EntityComponent({ groupRef = createRef<Group>(), entity, children }: Props) {
  useEffect(() => {
    if (!groupRef.current) return;

    //set the transform whenever it changes
    groupRef.current.position.fromArray(entity.transform.position);
    groupRef.current.scale.fromArray(entity.transform.scale);
    groupRef.current.rotation.setFromVector3(tempVec3.fromArray(entity.transform.rotation));
  }, [entity]);

  const Component = ENTITY_COMPONENTS[entity.type];

  return (
    <group ref={groupRef}>
      <Component {...(entity.props as any)} />

      {children}
    </group>
  );
}
