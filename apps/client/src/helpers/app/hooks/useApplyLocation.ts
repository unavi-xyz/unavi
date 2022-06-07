import { useFrame } from "@react-three/fiber";
import { RefObject } from "react";
import { Group } from "three";

import { VectorLocation } from "./useInterpolateLocation";

export function useApplyLocation(
  targetRef: RefObject<Group>,
  locationRef: RefObject<VectorLocation>
) {
  useFrame((_, delta) => {
    if (!targetRef.current || !locationRef.current) return;

    //apply position
    targetRef.current.position.copy(locationRef.current.position);

    //apply rotation
    targetRef.current.rotation.y = locationRef.current?.rotation;
  });
}
