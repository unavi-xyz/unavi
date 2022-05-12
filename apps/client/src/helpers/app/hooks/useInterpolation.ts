import { useFrame } from "@react-three/fiber";
import { MutableRefObject, RefObject, useEffect, useRef } from "react";
import { MathUtils, Group, Vector3 } from "three";

import { PUBLISH_INTERVAL } from "../constants";
import { Location } from "../types";

type VectorTransform = {
  position: Vector3;
  rotation: number;
};

export default function useInterpolation(
  ref: RefObject<Group>,
  transform: MutableRefObject<Location>
) {
  const idleRef = useRef(1);
  const walkRef = useRef(0);
  const runRef = useRef(0);
  const jumpRef = useRef(0);

  const deltaTotal = useRef(0);
  const tempVector3 = useRef(new Vector3());

  const real = useRef<VectorTransform>({
    position: new Vector3(),
    rotation: 0,
  });
  const prev = useRef<VectorTransform>({
    position: new Vector3(),
    rotation: 0,
  });

  useEffect(() => {
    const interval = setInterval(() => {
      if (!transform.current?.position || !transform.current?.rotation) return;

      prev.current.position.copy(real.current.position);
      prev.current.rotation = real.current.rotation;

      real.current.position.fromArray(transform.current.position);
      real.current.rotation = transform.current.rotation;

      deltaTotal.current = 0;
    }, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [transform]);

  useFrame((_, delta) => {
    if (!ref.current) return;

    deltaTotal.current += delta;
    const alpha = Math.min(deltaTotal.current * (1000 / PUBLISH_INTERVAL), 1);

    //animations
    const velocity = tempVector3.current
      .subVectors(real.current.position, ref.current.position)
      .divideScalar(delta);

    walkRef.current = Math.abs(velocity.x) + Math.abs(velocity.z);
    jumpRef.current = Math.abs(velocity.y);

    //position interp
    tempVector3.current.lerpVectors(
      prev.current.position,
      real.current.position,
      alpha
    );
    ref.current.position.copy(tempVector3.current);

    //rotation interp
    function normalize(angle: number) {
      while (angle < 0) {
        angle += 2 * Math.PI;
      }
      while (angle >= 2 * Math.PI) {
        angle -= 2 * Math.PI;
      }
      return angle;
    }

    const start = prev.current.rotation;
    let end = real.current.rotation;

    const forward = start - end;
    const backward = end - start;

    if (normalize(forward) < normalize(backward)) {
      if (end > start) end -= 2 * Math.PI;
    } else {
      if (end < start) end += 2 * Math.PI;
    }

    const rot = MathUtils.lerp(start, end, alpha);
    ref.current.rotation.y = rot;
  });

  // const weights: AnimationWeights = {
  //   idle: idleRef,
  //   walk: walkRef,
  //   run: runRef,
  //   jump: jumpRef,
  // };

  // return weights;
}
