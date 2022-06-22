import { useEffect, useRef, useState } from "react";
import { Group } from "three";

import { useGlobalTransform } from "../../../hooks/useGlobalTransform";
import { CylinderCollider } from "./CylinderCollider";
import { CylinderMesh } from "./CylinderMesh";

export interface CylinderProps {
  radiusTop?: number;
  radiusBottom?: number;
  height?: number;
  radialSegments?: number;
  openEnded?: boolean;
  physics?: boolean;
  materialId?: string;
}

export function Cylinder({
  radiusTop = 1,
  radiusBottom = 1,
  height = 1,
  radialSegments = 8,
  openEnded = false,
  physics = true,
  materialId,
}: CylinderProps) {
  const ref = useRef<Group>(null);
  const [key, setKey] = useState(0);

  const globalTransform = useGlobalTransform(ref);

  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [radiusTop, radiusBottom, height, , globalTransform]);

  return (
    <group ref={ref}>
      <CylinderMesh
        radiusTop={radiusTop}
        radiusBottom={radiusBottom}
        height={height}
        radialSegments={radialSegments}
        openEnded={openEnded}
        materialId={materialId}
      />

      {physics && (
        <CylinderCollider
          key={key}
          radiusTop={radiusTop}
          radiusBottom={radiusBottom}
          height={height}
          widthSegments={radialSegments}
          transform={globalTransform}
        />
      )}
    </group>
  );
}
