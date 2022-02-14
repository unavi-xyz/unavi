import { ASSET_NAMES } from "./assets";

export const PACKS: { [pack: string]: ASSET_NAMES[] } = {
  Basic: [ASSET_NAMES.Box, ASSET_NAMES.Sphere],
  Game: [ASSET_NAMES.Spawn],
};
