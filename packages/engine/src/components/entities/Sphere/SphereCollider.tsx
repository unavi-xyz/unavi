import { useSphere } from "@react-three/cannon";
import { useEffect } from "react";

import { Transform } from "../../../types";

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
