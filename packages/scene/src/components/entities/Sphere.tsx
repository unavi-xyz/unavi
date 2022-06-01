import { useSphere } from "@react-three/cannon";
import { useEffect, useRef, useState } from "react";
import { Mesh } from "three";

import { useGlobalTransform } from "../../hooks/useGlobalTransform";
import { useMaterial } from "../../hooks/useMaterial";
import { Transform } from "../../types";
import { Material } from "./Material";

export interface SphereProps {
  radius?: number;
  widthSegments?: number;
  heightSegments?: number;
  physics?: boolean;
  materialId?: string;
}

export function Sphere({
  radius = 0.5,
  widthSegments = 16,
  heightSegments = 16,
  physics = true,
  materialId,
}: SphereProps) {
  const ref = useRef<Mesh>(null);
  const [key, setKey] = useState(0);

  const material = useMaterial(materialId);
  const globalTransform = useGlobalTransform(ref);

  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [radius, globalTransform]);

  return (
    <mesh ref={ref}>
      <sphereBufferGeometry args={[radius, widthSegments, heightSegments]} />
      <Material {...material} />

      {physics && (
        <SphereCollider key={key} radius={radius} transform={globalTransform} />
      )}
    </mesh>
  );
}

export interface SphereColliderProps {
  radius: number;
  transform: Transform;
}

export function SphereCollider({ radius, transform }: SphereColliderProps) {
  const largestScale = Math.max(...transform.scale);
  const args: [number] = [radius * largestScale];

  const [ref, api] = useSphere(() => ({
    args,
    type: "Static",
    position: transform.position,
    rotation: transform.rotation,
  }));

  useEffect(() => {
    api.position.set(...transform.position);
    api.rotation.set(...transform.rotation);
  }, [transform]);

  return <object3D ref={ref} />;
}
