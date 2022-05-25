import Material from "./Material";
import { IMeshModuleComponent } from "./types";

interface ISphereMesh extends IMeshModuleComponent {
  radius: number;
  widthSegments: number;
  heightSegments: number;
}

export default function SphereMesh({
  radius,
  widthSegments,
  heightSegments,
  materialId,
}: ISphereMesh) {
  return (
    <mesh>
      <sphereBufferGeometry args={[radius, widthSegments, heightSegments]} />
      <Material materialId={materialId} />
    </mesh>
  );
}
