import { AssetName, Texture } from "./types";

import { Box } from "./assets/Box";
import { Sphere } from "./assets/Sphere";

interface Props {
  name: AssetName;
  params: any;
  textures: { [key: string]: Texture };
}

export function InstancedAsset({ name, params, textures }: Props) {
  switch (name) {
    case AssetName.Box:
      return <Box params={params} textures={textures} />;
    case AssetName.Sphere:
      return <Sphere params={params} textures={textures} />;
    default:
      return null;
  }
}
