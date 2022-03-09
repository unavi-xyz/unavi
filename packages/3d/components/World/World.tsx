import { Sky } from "@react-three/drei";

import { Scene } from "./types";
import { Ground } from "../Ground/Ground";
import AppInstance from "./AppInstance";

interface Props {
  scene: Scene;
}

export function World({ scene }: Props) {
  return (
    <group>
      <directionalLight intensity={0.7} position={[1, 2, 5]} />
      <ambientLight intensity={0.1} />
      <Sky inclination={1} />
      <Ground />

      {scene &&
        Object.values(scene).map((instance) => {
          return <AppInstance key={instance.id} instance={instance} />;
        })}
    </group>
  );
}
