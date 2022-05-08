interface ISphereMesh {
  radius: number;
}

export default function SphereMesh({ radius }: ISphereMesh) {
  return (
    <mesh>
      <sphereBufferGeometry args={[radius]} />
      <meshStandardMaterial color="red" />
    </mesh>
  );
}
