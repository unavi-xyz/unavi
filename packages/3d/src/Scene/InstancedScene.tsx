import { Sky } from "@react-three/drei";

import { Scene } from "./types";
import { InstancedObject } from "./InstancedObject";
import { Ground } from "../Ground/Ground";
import { SceneContext } from "./SceneContext";

interface Props {
  scene: Scene;
}

export function InstancedScene({ scene }: Props) {
  return (
    <group>
      <directionalLight intensity={0.7} position={[1, 2, 5]} />
      <ambientLight intensity={0.1} />
      <Sky inclination={1} />
      <Ground />

      <SceneContext.Provider
        value={{ assets: scene.assets, materials: scene.materials }}
      >
        {scene &&
          Object.values(scene.instances).map((instance) => {
            return <InstancedObject key={instance.id} instance={instance} />;
          })}
      </SceneContext.Provider>
    </group>
  );
}
