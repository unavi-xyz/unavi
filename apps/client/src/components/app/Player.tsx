import { Triplet, useSphere } from "@react-three/cannon";
import { PointerLockControls } from "@react-three/drei";
import { useFrame, useThree } from "@react-three/fiber";
import { useEffect, useRef } from "react";
import { Raycaster, Vector3 } from "three";

import { useKeyboardMovement } from "../../helpers/app/hooks/useKeyboardMovement";
import { useSpringVelocity } from "../../helpers/app/hooks/useSpringVelocity";

const PLAYER_HEIGHT = 1.6;
const PLAYER_SPEED = 5;
const JUMP_STRENGTH = 5;

const spawn: Triplet = [0, 2, 0];

export default function Player() {
  const position = useRef([0, 0, 0]);
  const velocity = useRef(new Vector3());
  const raycaster = useRef<Raycaster>();
  const jump = useRef(false);
  const grounded = useRef(false);

  const [ref, api] = useSphere(() => ({
    args: [PLAYER_HEIGHT / 2],
    type: "Dynamic",
    mass: 100,
    position: spawn,
    onCollide: (e) => {
      if (e.contact.contactNormal[1] > 0.6) {
        grounded.current = true;
      }
    },
  }));

  useEffect(() => {
    api.position.subscribe((v) => (position.current = v));
    api.velocity.subscribe((v) => velocity.current.fromArray(v));
  }, [api]);

  const { camera } = useThree();
  const { direction, updateVelocity } = useSpringVelocity(api, PLAYER_SPEED);

  useKeyboardMovement({ direction, jump });

  useFrame(() => {
    //teleport to spawn if fall into void
    if (camera.position.y < -30) {
      api.position.set(...spawn);
      velocity.current.set(0, 0, 0);
    }

    //set camera position
    camera.position.fromArray(position.current);
    camera.position.y += PLAYER_HEIGHT / 2;

    //jump
    if (jump.current && grounded.current) {
      grounded.current = false;
      velocity.current.y = JUMP_STRENGTH;
    }

    //apply velocity
    updateVelocity(camera, velocity.current);
  });

  return (
    <object3D ref={ref as any}>
      <raycaster
        ref={raycaster as any}
        near={PLAYER_HEIGHT - 0.01}
        far={PLAYER_HEIGHT + 0.1}
      />
      <PointerLockControls />
    </object3D>
  );
}
