import "@react-three/fiber"; // idk why i need this to stop type errors

import { SceneObject } from "..";

import { Asset, PROPERTIES } from "./classes/Asset";

import { Box } from "./basic/Box";
import { Sphere } from "./basic/Sphere";
import { Spawn } from "./game/Spawn";

const properties_basic = [
  PROPERTIES.position,
  PROPERTIES.rotation,
  PROPERTIES.scale,
  PROPERTIES.color,
  PROPERTIES.opacity,
];

export enum ASSET_NAMES {
  Box = "Box",
  Sphere = "Sphere",
  Spawn = "Spawn",
}

export const ASSETS: { [type: string]: Asset } = {
  Box: new Asset(ASSET_NAMES.Box, [...properties_basic]),
  Sphere: new Asset(ASSET_NAMES.Sphere, [...properties_basic]),
  Spawn: new Asset(ASSET_NAMES.Spawn, [PROPERTIES.position], 1),
};

export const PACKS: { [pack: string]: ASSET_NAMES[] } = {
  Basic: [ASSET_NAMES.Box, ASSET_NAMES.Sphere],
  Game: [ASSET_NAMES.Spawn],
};

export function getComponent(object: SceneObject) {
  switch (object.type) {
    case ASSET_NAMES.Box:
      return <Box object={object} />;
    case ASSET_NAMES.Sphere:
      return <Sphere object={object} />;
    case ASSET_NAMES.Spawn:
      return <Spawn />;
  }
}
