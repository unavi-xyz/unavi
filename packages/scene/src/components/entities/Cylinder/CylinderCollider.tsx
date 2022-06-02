import { CylinderArgs, useCylinder } from "@react-three/cannon";
import { useEffect } from "react";

import { Transform } from "../../../types";

export interface CylinderColliderProps {
  radiusTop: number;
  radiusBottom: number;
  height: number;
  widthSegments: number;
  transform: Transform;
}

export function CylinderCollider({
  radiusTop,
  radiusBottom,
  height,
  widthSegments,
  transform,
}: CylinderColliderProps) {
  const largestScaleX = Math.max(transform.scale[0], transform.scale[2]);

  const args: CylinderArgs = [
    radiusTop * largestScaleX,
    radiusBottom * largestScaleX,
    height * transform.scale[1],
    widthSegments,
  ];

  const [ref, api] = useCylinder(() => ({
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
