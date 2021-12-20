import * as THREE from "three";
import { useEffect, useRef } from "react";
import { useSphere } from "@react-three/cannon";
import { useFrame, useThree, Vector3 } from "@react-three/fiber";
import { PointerLockControls } from "@react-three/drei";

import { PHYSICS_GROUPS, VOID_LEVEL } from "../constants";
import { useSpringVelocity } from "./hooks";

import KeyboardMovement from "./controls/KeyboardMovement";
import Crosshair from "./Crosshair";
import { Triplet } from "../../../node_modules/@react-three/cannon/dist/hooks";

const PLAYER_HEIGHT = 1.6;
const PLAYER_SPEED = 2;
const SPHERE_RADIUS = 1;
const DOWN_VECTOR = new THREE.Vector3(0, -1, 0);
const HEIGHT_OFFSET = new THREE.Vector3(0, PLAYER_HEIGHT - SPHERE_RADIUS, 0);

interface Props {
  paused: boolean;
  spawn: Vector3;
}

export function Player({ paused = false, spawn = new THREE.Vector3(0, 2, 0) }) {
  const args: [number] = [SPHERE_RADIUS];

  const ray = useRef();
  const crosshair = useRef();
  const jump = useRef();
  const position = useRef(new THREE.Vector3());
  const velocity = useRef(new THREE.Vector3());
  const rotation = useRef(new THREE.Vector3());

  const { camera, scene, mouse } = useThree();

  const [ref, api] = useSphere(() => ({
    args,
    mass: 1,
    type: "Kinematic",
    position: spawn.toArray(),
    collisionFilterGroup: PHYSICS_GROUPS.PLAYER,
  }));

  const { direction, updateVelocity } = useSpringVelocity(api, PLAYER_SPEED);

  useEffect(() => {
    api.position.subscribe((p: Triplet) => position.current.fromArray(p));
    api.velocity.subscribe((v: Triplet) => velocity.current.fromArray(v));
  }, []);

  useFrame(() => {
    // if (position.current.y < VOID_LEVEL) {
    //   api.position.set(...spawn);
    // }

    //move hud
    // crosshair.current.rotation.copy(camera.rotation);
    // crosshair.current.position
    //   .copy(camera.position)
    //   .add(camera.getWorldDirection(rotation.current).multiplyScalar(1));

    //move camera
    camera.position.copy(position.current).add(HEIGHT_OFFSET);
    // console.log("👨‍🏫", position.current);

    //jumping
    // ray.current.set(camera.position, DOWN_VECTOR);
    // const intersects = ray.current.intersectObjects(scene.children);
    // const distance = intersects[0]?.distance;

    // if (jump.current && distance < PLAYER_HEIGHT + 0.1) {
    //   velocity.current.y = JUMP_STRENGTH;
    // }

    //apply velocity
    updateVelocity(camera, velocity.current);
  });

  return (
    <group>
      <mesh ref={ref} />

      <group ref={crosshair}>
        <Crosshair />
      </group>

      <KeyboardMovement paused={paused} direction={direction} jump={jump} />
      <PointerLockControls
        addEventListener={undefined}
        hasEventListener={undefined}
        removeEventListener={undefined}
        dispatchEvent={undefined}
      />
    </group>
  );
}
