import { ASSET_NAMES, Object, SceneObject } from "3d";

interface Props {
  object: SceneObject;
}

export default function PlayObject({ object }: Props) {
  return (
    <group>
      {object.params.type !== ASSET_NAMES.Spawn && <Object object={object} />}
    </group>
  );
}
