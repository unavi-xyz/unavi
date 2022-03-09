import { Box } from "./assets/Box";
import { AssetName, Instance } from "./types";

interface Props {
  instance: Instance;
}

export function InstancedAsset({ instance }: Props) {
  const { asset, params } = instance;

  switch (asset) {
    case AssetName.Box:
      return <Box params={params} />;
    default:
      return <group></group>;
  }
}
