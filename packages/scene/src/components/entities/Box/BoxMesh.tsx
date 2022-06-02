import { useMaterial } from "../../../hooks/useMaterial";
import { Material } from "../Material";

export interface BoxMeshProps {
  width: number;
  height: number;
  depth: number;
  materialId?: string;
}

export function BoxMesh({ width, height, depth, materialId }: BoxMeshProps) {
  const material = useMaterial(materialId);

  return (
    <mesh castShadow receiveShadow>
      <boxBufferGeometry args={[width, height, depth]} />
      <Material {...material} />
    </mesh>
  );
}
