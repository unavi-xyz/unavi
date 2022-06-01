import { Triplet } from "@react-three/cannon";
import { ComponentProps } from "react";

import { Box } from "./components/entities/Box";
import { Group } from "./components/entities/Group";
import { Sphere } from "./components/entities/Sphere";

//WELCOME TO THE MIND PALACE
//WHERE WE PROGRAM THINGS THAT DONT EXIST

//transform
export type Transform = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
};

//entity
export const ENTITY_COMPONENTS = {
  Group: Group,
  Box: Box,
  Sphere: Sphere,
};

export type EntityType = keyof typeof ENTITY_COMPONENTS;

export type Entity<T extends EntityType = EntityType> = {
  type: T;

  id: string;
  name?: string;

  parentId: string | null;
  children: Entity[];

  transform: Transform;
  props: ComponentProps<typeof ENTITY_COMPONENTS[T]>;
};

//assets
export type Image = string;

export type Material = {
  color: string;
  emissive: string;
  opacity: number;
  roughness: number;
  metalness: number;
  flatShading: boolean;
  textureId?: string;
};

export type ASSET_TYPES = {
  image: Image;
  material: Material;
};

export type AssetType = keyof ASSET_TYPES;

export type Asset<T extends AssetType = AssetType> = {
  type: T;
  uri: string;
  data?: ASSET_TYPES[T];
};

export type Assets = {
  [key: string]: Asset;
};

//scene
export type Scene = {
  tree: Entity;
  assets: Assets;
};
