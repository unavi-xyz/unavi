import { Asset, AssetName } from "./types";

import { boxDefaultParams } from "./assets/Box";
import { gltfDefaultParams } from "./assets/GLTF";
import { sphereDefaultParams } from "./assets/Sphere";

export const ASSETS = {
  Box: {
    name: AssetName.Box,
    params: boxDefaultParams,
  } as Asset,
  Sphere: {
    name: AssetName.Sphere,
    params: sphereDefaultParams,
  } as Asset,
  GLTF: {
    name: AssetName.GLTF,
    params: gltfDefaultParams,
  } as Asset,
};
