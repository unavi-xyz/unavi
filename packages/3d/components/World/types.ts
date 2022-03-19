import { Triplet } from "@react-three/cannon";

import { boxDefaultParams } from "./assets/Box";
import { gltfDefaultParams } from "./assets/GLTF";
import { sphereDefaultParams } from "./assets/Sphere";

export type Params = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
  radius: number;
  texture: string;
  model: string | null;
};

export enum AssetName {
  Box = "Box",
  Sphere = "Sphere",
  GLTF = "GLTF",
}

export type Asset = {
  name: AssetName;
  limit?: number;
  params: Partial<Params>;
};

export type Instance = {
  id: string;
  name: AssetName;
  params: Partial<Params>;
};

export type Texture = {
  name: string;
  value: string;
};

export type Model = {
  name: string;
  value: string;
};

export type Scene = {
  instances: {
    [key: string]: Instance;
  };
  textures: {
    [key: string]: Texture;
  };
  models: {
    [key: string]: Model;
  };
};

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
