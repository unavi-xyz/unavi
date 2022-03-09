import { grid_fragment, grid_vertex } from "./grid";

const THICKNESS = 0.1;

interface Props {
  size?: number;
}

export function Ground({ size = 10 }: Props) {
  return (
    <mesh position={[0, -THICKNESS / 2, 0]}>
      <boxGeometry args={[size + 0.00001, THICKNESS, size + 0.00001]} />
      <shaderMaterial
        vertexShader={grid_vertex}
        fragmentShader={grid_fragment}
      />
    </mesh>
  );
}
