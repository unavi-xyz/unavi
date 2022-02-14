import { ASSETS, ASSET_NAMES, SceneObject } from "..";

import { Box } from "./basic/Box";
import { Sphere } from "./basic/Sphere";
import { Spawn } from "./game/Spawn";

interface Props {
  instance: SceneObject<ASSET_NAMES>;
  editor?: boolean;
}

export function InstancedObject({ instance, editor = false }: Props) {
  switch (instance.type) {
    case ASSET_NAMES.Box:
      return (
        <Box
          params={instance.params as typeof ASSETS[ASSET_NAMES.Box]["params"]}
          editor={editor}
        />
      );
    case ASSET_NAMES.Sphere:
      return (
        <Sphere
          params={
            instance.params as typeof ASSETS[ASSET_NAMES.Sphere]["params"]
          }
          editor={editor}
        />
      );
    case ASSET_NAMES.Spawn:
      return <Spawn />;
    default:
      return <group></group>;
  }
}
