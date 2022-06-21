import { useFrame } from "@react-three/fiber";
import { RefObject, useRef } from "react";
import { Group, Vector3 } from "three";

import { AnimationWeights } from "@wired-xr/avatar";

import { JUMP_STRENGTH, PLAYER_SPEED } from "../../../components/app/Player";
import { VectorLocation } from "./useInterpolateLocation";

function normalizeWeight(weight: number) {
  return Math.min(Math.max(weight, 0), 1);
}

const JUMP_DURATION_SECONDS = 2;

export function useAnimationWeights(
  targetRef: RefObject<Group>,
  locationRef: RefObject<VectorLocation>
) {
  const prevPosition = useRef(new Vector3());
  const velocity = useRef(new Vector3());
  const averageVelocity = useRef(new Vector3());

  const idleRef = useRef(1);
  const walkRef = useRef(0);
  const runRef = useRef(0);
  const jumpRef = useRef(0);

  const jumpProgressRef = useRef(0);

  useFrame((_, delta) => {
    if (!targetRef.current || !locationRef.current) return;

    //get velocity
    velocity.current
      .set(0, 0, 0)
      .add(locationRef.current.position)
      .sub(prevPosition.current)
      .divideScalar(delta);

    //update position
    prevPosition.current.copy(targetRef.current.position);

    //average out velocity
    averageVelocity.current.add(velocity.current).divideScalar(2);

    //if positive y velocity, trigger jump
    if (
      averageVelocity.current.y > JUMP_STRENGTH / 2 &&
      jumpProgressRef.current === 0
    ) {
      jumpProgressRef.current = JUMP_DURATION_SECONDS;
    }

    //update jump progress
    jumpProgressRef.current = Math.max(jumpProgressRef.current - delta, 0);

    //set weights
    const x = Math.abs(averageVelocity.current.x) / PLAYER_SPEED;
    const y = Math.abs(averageVelocity.current.y) / JUMP_STRENGTH;
    const z = Math.abs(averageVelocity.current.z) / PLAYER_SPEED;

    walkRef.current = normalizeWeight(x + z - y);
    jumpRef.current = normalizeWeight(jumpProgressRef.current);
    idleRef.current = normalizeWeight(1 - walkRef.current - jumpRef.current);
  });

  const animationWeights: AnimationWeights = {
    idle: idleRef,
    walk: walkRef,
    run: runRef,
    jump: jumpRef,
  };

  return animationWeights;
}
