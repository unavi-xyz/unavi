import { RefObject, useEffect, useRef, useState } from "react";
import { Euler, Object3D, Quaternion, Vector3 } from "three";

import { DEFAULT_TRANSFORM } from "../constants";
import { areTransformsEqual } from "../utils";

export function useGlobalTransform(ref: RefObject<Object3D>) {
  const [transform, setTransform] = useState(DEFAULT_TRANSFORM);

  const tempVec3 = useRef(new Vector3());
  const tempQuat = useRef(new Quaternion());
  const tempEuler = useRef(new Euler());

  useEffect(() => {
    function updateTransform() {
      if (!ref.current) {
        const newTransform = DEFAULT_TRANSFORM;
        const isEqual = areTransformsEqual(newTransform, DEFAULT_TRANSFORM);
        if (!isEqual) setTransform(newTransform);
        return;
      }

      const position = ref.current.getWorldPosition(tempVec3.current).toArray();
      const scale = ref.current.getWorldScale(tempVec3.current).toArray();
      const rotation = tempVec3.current
        .setFromEuler(
          tempEuler.current.setFromQuaternion(ref.current.getWorldQuaternion(tempQuat.current))
        )
        .toArray();

      const newTransform = { position, rotation, scale };
      const isEqual = areTransformsEqual(newTransform, transform);
      if (!isEqual) setTransform(newTransform);
    }

    //update immediately
    const timeout = setTimeout(updateTransform, 0);
    //update on a timer
    const interval = setInterval(updateTransform, 500);

    return () => {
      clearTimeout(timeout);
      clearInterval(interval);
    };
  }, [ref, transform]);

  return transform;
}
