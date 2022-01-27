import "@react-three/fiber"; // idk why i need this to stop type errors

import { Asset, PROPERTIES } from "./classes/Asset";

const properties_basic = [
  PROPERTIES.position,
  PROPERTIES.rotation,
  PROPERTIES.scale,
  PROPERTIES.color,
  PROPERTIES.opacity,
  PROPERTIES.physics,
];

export enum ASSET_NAMES {
  Box = "Box",
  Sphere = "Sphere",
  Spawn = "Spawn",
}

export const ASSETS: { [type: string]: Asset } = {
  Box: new Asset(ASSET_NAMES.Box, [...properties_basic]),
  Sphere: new Asset(ASSET_NAMES.Sphere, [
    PROPERTIES.position,
    PROPERTIES.rotation,
    PROPERTIES.radius,
    PROPERTIES.color,
    PROPERTIES.opacity,
    PROPERTIES.physics,
  ]),
  Spawn: new Asset(ASSET_NAMES.Spawn, [PROPERTIES.position], 1),
};

export const PACKS: { [pack: string]: ASSET_NAMES[] } = {
  Basic: [ASSET_NAMES.Box, ASSET_NAMES.Sphere],
  Game: [ASSET_NAMES.Spawn],
};
