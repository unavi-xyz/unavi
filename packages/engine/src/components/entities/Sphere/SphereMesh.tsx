import { useMaterial } from "../../../hooks/useMaterial";
import { Material } from "../Material";

interface SphereMeshProps {
  radius: number;
  widthSegments: number;
  heightSegments: number;
  materialId?: string;
}

export function SphereMesh({
  radius,
  widthSegments,
  heightSegments,
  materialId,
}: SphereMeshProps) {
  const material = useMaterial(materialId);

  return (
    <mesh castShadow receiveShadow>
      <sphereBufferGeometry args={[radius, widthSegments, heightSegments]} />
      <Material {...material} />
    </mesh>
  );
}
