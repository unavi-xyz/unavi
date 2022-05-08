export interface IBoxMesh {
  width: number;
  height: number;
  depth: number;
}

export default function BoxMesh({ width, height, depth }: IBoxMesh) {
  return (
    <mesh>
      <boxBufferGeometry args={[width, height, depth]} />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
