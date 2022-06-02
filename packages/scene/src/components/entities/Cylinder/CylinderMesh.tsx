import { useMaterial } from "../../../hooks/useMaterial";
import { Material } from "../Material";

export interface CylinderMeshProps {
  radiusTop: number;
  radiusBottom: number;
  height: number;
  radialSegments: number;
  openEnded: boolean;
  materialId?: string;
}

export function CylinderMesh({
  radiusTop,
  radiusBottom,
  height,
  radialSegments,
  openEnded,
  materialId,
}: CylinderMeshProps) {
  const material = useMaterial(materialId);

  return (
    <mesh>
      <cylinderBufferGeometry
        args={[radiusTop, radiusBottom, height, radialSegments, 1, openEnded]}
      />

      <Material {...material} />
    </mesh>
  );
}
