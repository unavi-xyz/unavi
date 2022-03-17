import { Sky } from "@react-three/drei";

import { Model, Scene, Texture } from "./types";
import { InstancedAsset } from "./InstancedAsset";
import { Ground } from "../Ground/Ground";

interface Props {
  scene: Scene;
  textures?: { [key: string]: Texture };
  models?: { [key: string]: Model };
}

export function World({ scene, textures, models }: Props) {
  return (
    <group>
      <directionalLight intensity={0.7} position={[1, 2, 5]} />
      <ambientLight intensity={0.1} />
      <Sky inclination={1} />
      <Ground />

      {scene &&
        Object.values(scene.instances).map((instance) => {
          return (
            <InstancedAsset
              key={instance.id}
              name={instance.name}
              params={instance.params}
              textures={textures ?? scene.textures}
              models={models ?? scene.models}
            />
          );
        })}
    </group>
  );
}
