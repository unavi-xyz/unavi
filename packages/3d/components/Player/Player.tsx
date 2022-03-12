import { MutableRefObject, useEffect, useRef } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { Triplet, useSphere } from "@react-three/cannon";
import { PointerLockControls } from "@react-three/drei";
import { Group, Raycaster, Vector3 } from "three";

import { useKeyboardMovement } from "./useKeyboardMovement";
import { useSpringVelocity } from "./useSpringVelocity";

const PLAYER_HEIGHT = 1.6;

const PLAYER_SPEED = 5;
const HEIGHT_OFFSET = new Vector3(0, PLAYER_HEIGHT / 2, 0);

interface Props {
  spawn?: Vector3;
}

export function Player({ spawn = new Vector3(0, 2, 0) }: Props) {
  const position = useRef(new Vector3().copy(spawn));
  const velocity = useRef(new Vector3());
  const raycaster = useRef<Raycaster>();
  const grounded = useRef(false);
  const jump = useRef(false);

  const [ref, api] = useSphere(() => ({
    args: [PLAYER_HEIGHT / 2],
    type: "Dynamic",
    mass: 60,
    position: spawn.add(HEIGHT_OFFSET).toArray(),
    onCollide: (e) => {
      if (e.contact.contactNormal[1] > 0.6) {
        grounded.current = true;
      }
    },
  }));

  const { camera } = useThree();
  const { direction, updateVelocity } = useSpringVelocity(api, PLAYER_SPEED);

  useKeyboardMovement({ direction, jump });

  useEffect(() => {
    api.position.subscribe((p: Triplet) => position.current.fromArray(p));
    api.velocity.subscribe((v: Triplet) => velocity.current.fromArray(v));
  }, [api.position, api.velocity]);

  useFrame(() => {
    //teleport to spawn if fall into void
    if (camera.position.y < -30) {
      api.position.copy(spawn);
      velocity.current.set(0, 0, 0);
    }

    //move camera
    camera.position.copy(position.current).add(HEIGHT_OFFSET);

    //jump
    if (jump.current && grounded.current) {
      grounded.current = false;
      velocity.current.y = 6;
    }

    updateVelocity(camera, velocity.current);
  });

  return (
    <group ref={ref}>
      <raycaster
        near={PLAYER_HEIGHT - 0.01}
        far={PLAYER_HEIGHT + 0.1}
        ref={raycaster}
      />
      <PointerLockControls />
    </group>
  );
}
