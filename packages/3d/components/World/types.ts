import { Triplet } from "@react-three/cannon";

export type Material = {
  color: string;
  opacity: number;
  texture: string | undefined;
};

export type Params = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
  radius: number;
  model: string | undefined;
  material: Material;
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
