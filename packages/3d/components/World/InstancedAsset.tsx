import { AssetName, Model, Texture } from "./types";

import { Box } from "./assets/Box";
import { Sphere } from "./assets/Sphere";
import { GLTFModel } from "./assets/GLTF";

interface Props {
  name: AssetName;
  params: any;
  textures: { [key: string]: Texture };
  models: { [key: string]: Model };
}

export function InstancedAsset({ name, params, textures, models }: Props) {
  switch (name) {
    case AssetName.Box:
      return <Box params={params} textures={textures} />;
    case AssetName.Sphere:
      return <Sphere params={params} textures={textures} />;
    case AssetName.GLTF:
      return <GLTFModel params={params} models={models} />;
    default:
      return null;
  }
}
