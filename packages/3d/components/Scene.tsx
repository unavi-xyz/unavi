import { Sky } from "@react-three/drei";
import { Scene } from "ceramic";
import { ASSET_NAMES, Ground, InstancedObject, SceneObject } from "..";

interface Props {
  scene: SceneObject[];
}

export function Scene({ scene }: Props) {
  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Ground />
      <Sky />

      {scene?.map((params: SceneObject, i) => {
        if (params.type === ASSET_NAMES.Spawn) return <group></group>;
        return <InstancedObject key={i} params={params} />;
      })}
    </group>
  );
}
