import { Triplet, useBox } from "@react-three/cannon";
import { grid_vertex, grid_fragment } from "../shaders/grid";

const THICKNESS = 0.1;

export function Ground({ size = 20 }) {
  const args: Triplet = [size + 0.01, THICKNESS, size + 0.01];
  const position: Triplet = [0, THICKNESS / -2, 0];

  const [ref] = useBox(() => ({
    args,
    position,
  }));

  return (
    <mesh ref={ref}>
      <boxGeometry args={args} />
      <shaderMaterial
        vertexShader={grid_vertex}
        fragmentShader={grid_fragment}
      />
    </mesh>
  );
}
