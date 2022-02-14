import { useContext, useEffect, useRef } from "react";
import { Triplet, useBox } from "@react-three/cannon";
import { useFrame, useThree } from "@react-three/fiber";
import { PointerLockControls } from "@react-three/drei";
import { Raycaster, Vector3 } from "three";

import { PHYSICS_GROUPS, PUBLISH_INTERVAL, VOID_LEVEL } from "../..";
import { useSpringVelocity } from "./hooks/useSpringVelocity";
import { MultiplayerContext } from "../../contexts/MultiplayerContext";

import KeyboardMovement from "./controls/KeyboardMovement";

const PLAYER_HEIGHT = 1.6;
const PLAYER_WIDTH = 0.8;
const PLAYER_SPEED = 5;
const JUMP_STRENGTH = 3;

const DOWN_VECTOR = new Vector3(0, -1, 0);
const HEIGHT_OFFSET = new Vector3(0, PLAYER_HEIGHT / 2, 0);

interface Props {
  spawn?: Triplet;
}

export function Player({ spawn = [0, 0, 0] }: Props) {
  const { publishLocation } = useContext(MultiplayerContext);

  const downRay = useRef<undefined | Raycaster>();
  const jump = useRef(false);
  const position = useRef(new Vector3().fromArray(spawn));
  const velocity = useRef(new Vector3());

  const { camera } = useThree();

  const [ref, api] = useBox(() => ({
    args: [PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH],
    type: "Kinematic",
    mass: 1,
    position: spawn,
    collisionFilterGroup: PHYSICS_GROUPS.PLAYER,
  }));

  const { direction, updateVelocity } = useSpringVelocity(api, PLAYER_SPEED);

  useEffect(() => {
    function publishPosition() {
      if (!publishLocation) return;

      const adjustedPos = position.current.toArray();
      adjustedPos[1] -= PLAYER_HEIGHT / 2;

      const rot = camera.getWorldDirection(new Vector3());
      const sign = Math.sign(rot.x);
      const angle = Math.PI - (Math.atan(rot.z / rot.x) - (Math.PI / 2) * sign);

      publishLocation(adjustedPos, angle);
    }

    api.position.subscribe((p: Triplet) => position.current.fromArray(p));
    api.velocity.subscribe((v: Triplet) => velocity.current.fromArray(v));

    const interval = setInterval(publishPosition, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [api.position, api.velocity, publishLocation]);

  useFrame((state, delta) => {
    if (position.current.y < VOID_LEVEL) {
      api.position.set(...spawn);
      velocity.current.set(0, 0, 0);
    }

    //move camera
    camera.position.copy(position.current).add(HEIGHT_OFFSET);

    //ground detection
    if (downRay.current) {
      downRay.current.set(camera.position, DOWN_VECTOR);

      const intersects = downRay.current.intersectObject(state.scene);
      const distance = intersects[0]?.distance;

      if (distance < PLAYER_HEIGHT + 0.1) {
        velocity.current.y = 0;

        if (distance < PLAYER_HEIGHT) {
          position.current.y += PLAYER_HEIGHT - distance;
          api.position.copy(position.current);
        }

        //jumping
        if (jump.current) velocity.current.y = JUMP_STRENGTH;
      } else {
        //gravity
        velocity.current.y -= 9.8 * delta;
      }
    }

    //apply velocity
    updateVelocity(camera, velocity.current);
  });

  return (
    <group ref={ref}>
      <raycaster ref={downRay} />

      <KeyboardMovement direction={direction} jump={jump} />
      <PointerLockControls />
    </group>
  );
}
