import Ground from "./Ground";

export default function World() {
  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Ground />
    </group>
  );
}
