import { useEffect, useState } from "react";
import { Euler, Group, Quaternion, Vector3 } from "three";

import { DEFAULT_TRANSFORM } from "../../../constants";
import { COLLIDER_COMPONENTS } from "../../../modules";
import { Entity, IModule } from "../../../types";

const tempVector3 = new Vector3();
const tempQuaternion = new Quaternion();
const tempEuler = new Euler();

interface Props {
  module: IModule;
  entity: Entity;
  entityRef: React.RefObject<Group>;
}

export default function ColliderModule({ module, entity, entityRef }: Props) {
  const [key, setKey] = useState(0);
  const [absoluteTransform, setAbsoluteTransform] = useState(DEFAULT_TRANSFORM);

  useEffect(() => {
    const interval = setInterval(() => {
      const matrix = entityRef.current?.matrixWorld;
      if (!matrix) return;

      const position = entityRef.current
        .getWorldPosition(tempVector3)
        .toArray();
      const rotation = tempVector3
        .setFromEuler(
          tempEuler.setFromQuaternion(
            entityRef.current.getWorldQuaternion(tempQuaternion)
          )
        )
        .toArray();
      const scale = entityRef.current.getWorldScale(tempVector3).toArray();

      // 🙃
      if (
        position[0] !== absoluteTransform.position[0] ||
        position[1] !== absoluteTransform.position[1] ||
        position[2] !== absoluteTransform.position[2] ||
        rotation[0] !== absoluteTransform.rotation[0] ||
        rotation[1] !== absoluteTransform.rotation[1] ||
        rotation[2] !== absoluteTransform.rotation[2] ||
        scale[0] !== absoluteTransform.scale[0] ||
        scale[1] !== absoluteTransform.scale[1] ||
        scale[2] !== absoluteTransform.scale[2]
      ) {
        setAbsoluteTransform({
          position,
          rotation,
          scale,
        });
      }
    }, 100);

    return () => clearInterval(interval);
  }, [entity, entityRef, absoluteTransform]);

  //cannon physics objects cant change args once created
  //remount the component every time they change
  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [entity, absoluteTransform]);

  const Component = COLLIDER_COMPONENTS[module.variation];
  return (
    <Component
      key={key}
      {...(module.props as any)}
      transform={absoluteTransform}
    />
  );
}
