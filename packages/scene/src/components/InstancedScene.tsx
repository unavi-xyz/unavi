import { Physics } from "@react-three/cannon";
import { Sky } from "@react-three/drei";

import { Scene } from "../types";
import { InstancedEntity } from "./InstancedEntity";
import { MaterialProvider } from "./MaterialProvider";

interface Props {
  scene: Scene;
  children?: React.ReactNode;
}

export function InstancedScene({ scene, children }: Props) {
  return (
    <Physics>
      <ambientLight intensity={0.2} />
      <directionalLight intensity={1} position={[-1, 1.5, -2]} />
      <Sky />

      <MaterialProvider materials={scene.materials}>
        <InstancedEntity entity={scene.tree} />
      </MaterialProvider>

      <group>{children}</group>
    </Physics>
  );
}
