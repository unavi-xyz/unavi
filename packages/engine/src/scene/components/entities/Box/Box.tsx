import { useEffect, useRef, useState } from "react";
import { Group } from "three";

import { useGlobalTransform } from "../../../hooks/useGlobalTransform";
import { BoxCollider } from "./BoxCollider";
import { BoxMesh } from "./BoxMesh";

export interface BoxProps {
  width?: number;
  height?: number;
  depth?: number;
  physics?: boolean;
  materialId?: string;
}

export function Box({ width = 1, height = 1, depth = 1, physics = true, materialId }: BoxProps) {
  const ref = useRef<Group>(null);
  const [key, setKey] = useState(0);

  const globalTransform = useGlobalTransform(ref);

  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [width, height, depth, globalTransform]);

  return (
    <group ref={ref}>
      <BoxMesh width={width} height={height} depth={depth} materialId={materialId} />

      {physics && (
        <BoxCollider
          key={key}
          width={width}
          height={height}
          depth={depth}
          transform={globalTransform}
        />
      )}
    </group>
  );
}
