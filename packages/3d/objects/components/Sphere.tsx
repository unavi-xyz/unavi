import { SceneObject } from "../..";

interface Props {
  object: SceneObject;
}

export function Sphere({ object }: Props) {
  const args: [number] = [0.5];

  return (
    <mesh>
      <sphereGeometry args={args} />
      <meshPhongMaterial
        color={object.color}
        opacity={object.opacity}
        transparent={Boolean(object.opacity < 1)}
      />
    </mesh>
  );
}
