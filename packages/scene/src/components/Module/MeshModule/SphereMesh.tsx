import Material from "./Material";
import { IMeshModuleComponent } from "./types";

interface ISphereMesh extends IMeshModuleComponent {
  radius: number;
}

export default function SphereMesh({ radius, materialId }: ISphereMesh) {
  return (
    <mesh>
      <sphereBufferGeometry args={[radius]} />
      <Material materialId={materialId} />
    </mesh>
  );
}
