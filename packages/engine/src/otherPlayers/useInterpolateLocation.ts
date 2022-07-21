import { useFrame } from "@react-three/fiber";
import { RefObject, useEffect, useRef } from "react";
import { MathUtils, Vector3 } from "three";

import { LOCATION_PUBLISH_INTERVAL_MS } from "../networking/constants";
import { PlayerLocation } from "../networking/types";

function normalizeAngle(angle: number) {
  while (angle < 0) {
    angle += 2 * Math.PI;
  }
  while (angle >= 2 * Math.PI) {
    angle -= 2 * Math.PI;
  }
  return angle;
}

export type VectorLocation = {
  position: Vector3;
  rotation: number;
};

//read location on an interval
//interpoalte it between intervals
export function useInterpolateLocation(locationRef: RefObject<PlayerLocation>) {
  const deltaTotal = useRef(0);

  const prevRef = useRef<VectorLocation>({
    position: new Vector3(),
    rotation: 0,
  });
  const targetRef = useRef<VectorLocation>({
    position: new Vector3(),
    rotation: 0,
  });
  const interpolatedRef = useRef<VectorLocation>({
    position: new Vector3(),
    rotation: 0,
  });

  useEffect(() => {
    //read location on an interval
    const interval = setInterval(() => {
      if (!locationRef.current) return;

      prevRef.current.position.copy(targetRef.current.position);
      prevRef.current.rotation = targetRef.current.rotation;

      targetRef.current.position.fromArray(locationRef.current.position);
      targetRef.current.rotation = locationRef.current.rotation;

      deltaTotal.current = 0;
    }, LOCATION_PUBLISH_INTERVAL_MS);

    return () => {
      clearInterval(interval);
    };
  }, [locationRef]);

  //interpoalte it between intervals
  useFrame((_, delta) => {
    deltaTotal.current += delta;
    const alpha = Math.min(deltaTotal.current * (1000 / LOCATION_PUBLISH_INTERVAL_MS), 1);

    //interpolate position
    interpolatedRef.current.position.lerpVectors(
      prevRef.current.position,
      targetRef.current.position,
      alpha
    );

    //normalize rotation
    const start = prevRef.current.rotation;
    let end = targetRef.current.rotation;

    const forward = start - end;
    const backward = end - start;

    if (normalizeAngle(forward) < normalizeAngle(backward)) {
      if (end > start) end -= 2 * Math.PI;
    } else {
      if (end < start) end += 2 * Math.PI;
    }

    //interpolate rotation
    interpolatedRef.current.rotation = MathUtils.lerp(start, end, alpha);
  });

  return interpolatedRef;
}
