import { Sky } from "@react-three/drei";
import { Scene } from "ceramic";
import { ASSET_NAMES, Ground, InstancedObject, SceneObject } from "..";

interface Props {
  objects: SceneObject[];
}

export function Scene({ objects }: Props) {
  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Ground />
      <Sky />

      {objects?.map((object: SceneObject) => {
        if (object.type === ASSET_NAMES.Spawn) return <group></group>;
        return <InstancedObject key={object.id} object={object} />;
      })}
    </group>
  );
}
