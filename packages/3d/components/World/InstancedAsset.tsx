import { AssetName } from "./types";

import { Box } from "./assets/Box";
import { Sphere } from "./assets/Sphere";

interface Props {
  name: AssetName;
  params: any;
}

export function InstancedAsset({ name, params }: Props) {
  switch (name) {
    case AssetName.Box:
      return <Box params={params} />;
    case AssetName.Sphere:
      return <Sphere params={params} />;
    default:
      return null;
  }
}
