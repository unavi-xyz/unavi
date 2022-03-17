import { Sky } from "@react-three/drei";

import { Scene, Texture } from "./types";
import { InstancedAsset } from "./InstancedAsset";
import { Ground } from "../Ground/Ground";

interface Props {
  scene: Scene;
  textures?: { [key: string]: Texture };
}

export function World({ scene, textures }: Props) {
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
            />
          );
        })}
    </group>
  );
}
