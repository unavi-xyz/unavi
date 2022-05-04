import { Debug, Physics } from "@react-three/cannon";
import { Sky } from "@react-three/drei";

import { Scene } from "../types";
import { InstancedEntity } from "./InstancedEntity";

interface Props {
  scene: Scene;
}

export function InstancedScene({ scene }: Props) {
  return (
    <group>
      <ambientLight intensity={0.2} />
      <directionalLight intensity={1} position={[-1, 1.5, -2]} />
      <Sky />

      <Physics>
        <Debug>
          <InstancedEntity entity={scene.tree} />
        </Debug>
      </Physics>
    </group>
  );
}
