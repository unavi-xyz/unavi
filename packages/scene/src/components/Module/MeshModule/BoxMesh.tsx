import Material from "./Material";
import { IMeshModuleComponent } from "./types";

export interface IBoxMesh extends IMeshModuleComponent {
  width: number;
  height: number;
  depth: number;
}

export default function BoxMesh({
  width,
  height,
  depth,
  materialId,
}: IBoxMesh) {
  return (
    <mesh>
      <boxBufferGeometry args={[width, height, depth]} />
      <Material materialId={materialId} />
    </mesh>
  );
}
