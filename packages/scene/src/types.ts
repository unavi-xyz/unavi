import { Triplet } from "@react-three/cannon";
import { ComponentProps } from "react";

import { AmbientLight } from "./components/entities/AmbientLight";
import { Box } from "./components/entities/Box/Box";
import { Cylinder } from "./components/entities/Cylinder/Cylinder";
import { DirectionalLight } from "./components/entities/DirectionalLight";
import { Group } from "./components/entities/Group";
import { Model } from "./components/entities/Model/Model";
import { PointLight } from "./components/entities/PointLight";
import { Sphere } from "./components/entities/Sphere/Sphere";
import { SpotLight } from "./components/entities/SpotLight";

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
  Cylinder: Cylinder,
  Model: Model,

  PointLight: PointLight,
  AmbientLight: AmbientLight,
  DirectionalLight: DirectionalLight,
  SpotLight: SpotLight,
};

export type EntityType = keyof typeof ENTITY_COMPONENTS;

export interface Entity<T extends EntityType = EntityType> {
  type: T;

  id: string;
  name?: string;

  parentId: string | null;
  children: Entity[];

  transform: Transform;
  props: ComponentProps<typeof ENTITY_COMPONENTS[T]>;
}

//assets

export interface IMaterial {
  color: string;
  emissive: string;
  opacity: number;
  roughness: number;
  metalness: number;
  flatShading: boolean;
  side: "Front" | "Back" | "Double";
  textureId?: string;
}

export interface ASSET_TYPES {
  image: string;
  material: IMaterial;
  model: string;
}

export type AssetType = keyof ASSET_TYPES;

export interface Asset<T extends AssetType = AssetType> {
  type: T;
  uri: string;
  data?: ASSET_TYPES[T];
}

export interface Assets {
  [key: string]: Asset;
}

//scene
export interface Scene {
  tree: Entity;
  assets: Assets;
}
