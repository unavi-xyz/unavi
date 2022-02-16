import { Asset } from "./classes/Asset";

import { BoxDefault } from "./basic/Box";
import { SphereDefault } from "./basic/Sphere";
import { SpawnDefault } from "./game/Spawn";

export enum ASSET_NAMES {
  Box = "Box",
  Sphere = "Sphere",
  Spawn = "Spawn",
}

export const ASSETS = {
  [ASSET_NAMES.Box]: new Asset<typeof BoxDefault>(ASSET_NAMES.Box, BoxDefault),
  [ASSET_NAMES.Sphere]: new Asset<typeof SphereDefault>(
    ASSET_NAMES.Sphere,
    SphereDefault
  ),
  [ASSET_NAMES.Spawn]: new Asset<typeof SpawnDefault>(
    ASSET_NAMES.Spawn,
    SpawnDefault,
    { limit: 1, hidden: true }
  ),
};
