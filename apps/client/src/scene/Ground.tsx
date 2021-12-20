import * as THREE from "three";
import { useEffect } from "react";
import { Triplet, useBox } from "@react-three/cannon";

import { PHYSICS_GROUPS } from "../constants";
import { grid_vertex, grid_fragment } from "../shaders/grid";

export default function Ground() {
  const args: Triplet = [50, 0.1, 50];
  const position = new THREE.Vector3();

  const [ref, api] = useBox(() => ({
    args,
    position,
    collisionFilterGroup: PHYSICS_GROUPS.WORLD,
  }));

  useEffect(() => {
    api.position.copy(position);
  }, [position]);

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <shaderMaterial
        vertexShader={grid_vertex}
        fragmentShader={grid_fragment}
      />
    </mesh>
  );
}
