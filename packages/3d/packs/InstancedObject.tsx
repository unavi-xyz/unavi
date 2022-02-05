import { ASSET_NAMES, SceneObject } from ".";
import { Box } from "./basic/Box";
import { Sphere } from "./basic/Sphere";
import { Spawn } from "./game/Spawn";

interface Props {
  object: SceneObject;
  editor?: boolean;
}

export function InstancedObject({ object, editor = false }: Props) {
  switch (object.type) {
    case ASSET_NAMES.Box:
      return <Box object={object} editor={editor} />;
    case ASSET_NAMES.Sphere:
      return <Sphere object={object} editor={editor} />;
    case ASSET_NAMES.Spawn:
      return <Spawn />;
    default:
      return <group></group>;
  }
}
