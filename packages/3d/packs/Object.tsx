import { ASSET_NAMES, Params } from ".";
import { Box } from "./basic/Box";
import { Sphere } from "./basic/Sphere";
import { Spawn } from "./game/Spawn";

interface Props {
  params: Params;
  editor?: boolean;
}

export function Object({ params, editor = false }: Props) {
  switch (params.type) {
    case ASSET_NAMES.Box:
      return <Box params={params} editor={editor} />;
    case ASSET_NAMES.Sphere:
      return <Sphere params={params} editor={editor} />;
    case ASSET_NAMES.Spawn:
      return <Spawn />;
    default:
      return <group></group>;
  }
}
