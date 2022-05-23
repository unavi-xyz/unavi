import { useEffect } from "react";
import { useState } from "react";

import { useMaterial } from "../../MaterialProvider";

interface Props {
  materialId: string | undefined;
}

export default function Material({ materialId }: Props) {
  const [key, setKey] = useState(0);

  const material = useMaterial(materialId);

  const transparent = Boolean((material?.opacity ?? 1) < 1);

  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [material]);

  return (
    <meshStandardMaterial
      key={key}
      color={material?.color ?? "#ffffff"}
      emissive={material?.emissive}
      roughness={material?.roughness}
      metalness={material?.metalness}
      opacity={material?.opacity}
      transparent={transparent}
      flatShading={material?.flatShading}
    />
  );
}
