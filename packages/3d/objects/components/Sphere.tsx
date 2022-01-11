export function Sphere() {
  const args: [number] = [0.5];

  return (
    <mesh>
      <sphereGeometry args={args} />
      <meshPhongMaterial />
    </mesh>
  );
}
