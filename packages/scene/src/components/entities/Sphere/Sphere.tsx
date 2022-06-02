import { useEffect, useRef, useState } from "react";
import { Group } from "three";

import { useGlobalTransform } from "../../../hooks/useGlobalTransform";
import { SphereMesh } from "./SphereMesh";
import { SphereCollider } from "./Spherecollider";

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
  const ref = useRef<Group>(null);
  const [key, setKey] = useState(0);

  const globalTransform = useGlobalTransform(ref);

  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [radius, globalTransform]);

  return (
    <group ref={ref}>
      <SphereMesh
        radius={radius}
        widthSegments={widthSegments}
        heightSegments={heightSegments}
        materialId={materialId}
      />

      {physics && (
        <SphereCollider key={key} radius={radius} transform={globalTransform} />
      )}
    </group>
  );
}
