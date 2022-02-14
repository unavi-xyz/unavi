import { ASSET_NAMES, InstancedObject, SceneObject } from "../..";

interface Props {
  objects: SceneObject<ASSET_NAMES>[];
}

export function Objects({ objects }: Props) {
  return (
    <group>
      {objects?.map((object: SceneObject<ASSET_NAMES>, i) => {
        if (object.type === ASSET_NAMES.Spawn) return <group key={i}></group>;
        return <InstancedObject key={object.id} instance={object} />;
      })}
    </group>
  );
}
