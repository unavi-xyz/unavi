import { Triplet, useBox } from "@react-three/cannon";
import { grid_fragment, grid_vertex } from "./grid";

const THICKNESS = 0.1;

interface Props {
  size?: number;
}

export function Ground({ size = 10 }: Props) {
  const args: Triplet = [size + 0.00001, THICKNESS, size + 0.00001];

  const [ref] = useBox(() => ({
    args,
    type: "Static",
    position: [0, -THICKNESS / 2, 0],
  }));

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
