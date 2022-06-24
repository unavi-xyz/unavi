import { Triplet, useBox } from "@react-three/cannon";
import { useEffect } from "react";

import { Transform } from "../../../types";

export interface BoxColliderProps {
  width: number;
  height: number;
  depth: number;
  transform: Transform;
}

export function BoxCollider({
  width,
  height,
  depth,
  transform,
}: BoxColliderProps) {
  const args: Triplet = [
    width * transform.scale[0],
    height * transform.scale[1],
    depth * transform.scale[2],
  ];

  const [ref, api] = useBox(() => ({
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
