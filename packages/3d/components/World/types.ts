import { Triplet } from "@react-three/cannon";

export type Params = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
};

export enum AssetName {
  Box = "Box",
  Sphere = "Sphere",
}

export type Asset = {
  name: AssetName;
  limit?: number;
  params: Partial<Params>;
};

export type Instance = {
  id: string;
  asset: AssetName;
  params: Partial<Params>;
};

export type Scene = {
  [key: string]: Instance;
};
