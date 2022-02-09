import { Sky } from "@react-three/drei";
import { Scene } from "ceramic";
import { ASSET_NAMES, Ground, InstancedObject, SceneObject } from "../..";

interface Props {
  objects: SceneObject<ASSET_NAMES>[];
}

export function Scene({ objects }: Props) {
  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Ground />
      <Sky />

      {objects?.map((object: SceneObject<ASSET_NAMES>, i) => {
        if (object.type === ASSET_NAMES.Spawn) return <group key={i}></group>;
        return <InstancedObject key={object.id} instance={object} />;
      })}
    </group>
  );
}
