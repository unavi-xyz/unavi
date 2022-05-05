import { Debug, Physics } from "@react-three/cannon";
import { Sky } from "@react-three/drei";

import { Scene } from "../types";
import { InstancedEntity } from "./InstancedEntity";

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

      <Debug>
        <InstancedEntity entity={scene.tree} />
      </Debug>

      <group>{children}</group>
    </Physics>
  );
}
