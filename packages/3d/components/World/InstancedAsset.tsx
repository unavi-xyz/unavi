import { AssetName, Instance } from "./types";

import { Box } from "./assets/Box";
import { Sphere } from "./assets/Sphere";

interface Props {
  instance: Instance;
}

export function InstancedAsset({ instance }: Props) {
  const { name, params } = instance;

  switch (name) {
    case AssetName.Box:
      return <Box params={params as any} />;
    case AssetName.Sphere:
      return <Sphere params={params as any} />;
    default:
      return <group></group>;
  }
}
