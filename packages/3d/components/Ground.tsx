import { Triplet } from "@react-three/cannon";
import { grid_vertex, grid_fragment } from "../shaders/grid";

const THICKNESS = 0.1;

export function Ground({ size = 20 }) {
  const args: Triplet = [size + 0.01, THICKNESS, size + 0.01];

  return (
    <mesh position={[0, THICKNESS / -2, 0]}>
      <boxBufferGeometry args={args} />
      <shaderMaterial
        vertexShader={grid_vertex}
        fragmentShader={grid_fragment}
      />
    </mesh>
  );
}
