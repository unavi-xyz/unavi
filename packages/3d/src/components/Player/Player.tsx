import { useEffect, useRef } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { Triplet, useBox } from "@react-three/cannon";
import { PointerLockControls } from "@react-three/drei";
import { Vector3 } from "three";

import useKeyboardMovement from "./useKeyboardMovement";
import { useSpringVelocity } from "./useSpringVelocity";

const PLAYER_WIDTH = 0.5;
const PLAYER_HEIGHT = 1.6;

const PLAYER_SPEED = 5;
const HEIGHT_OFFSET = new Vector3(0, PLAYER_HEIGHT * 0.75, 0);

export function Player() {
  const position = useRef(new Vector3());
  const velocity = useRef(new Vector3());

  const [ref, api] = useBox(() => ({
    args: [PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH],
    type: "Kinematic",
    mass: 1,
    position: [0, 0, 0],
  }));

  const { camera } = useThree();
  const { direction, updateVelocity } = useSpringVelocity(api, PLAYER_SPEED);

  useKeyboardMovement({ direction });

  useEffect(() => {
    api.position.subscribe((p: Triplet) => position.current.fromArray(p));
    api.velocity.subscribe((v: Triplet) => velocity.current.fromArray(v));
  }, [api.position, api.velocity]);

  useFrame(() => {
    camera.position.copy(position.current).add(HEIGHT_OFFSET);
    updateVelocity(camera, velocity.current);
  });

  return (
    <group ref={ref}>
      <PointerLockControls />
    </group>
  );
}
