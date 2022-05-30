import { Triplet } from "@react-three/cannon";
import { ComponentProps } from "react";

import {
  COLLIDER_COMPONENTS,
  ColliderType,
  MESH_COMPONENTS,
  MeshType,
  ModuleType,
} from "./modules";

//WELCOME TO THE MIND PALACE
//WHERE WE PROGRAM THINGS THAT DONT EXIST

//data
export type Transform = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
};

//module
export interface BaseModule<T extends ModuleType = ModuleType> {
  id: string;
  type: T;
  variation: string;
  props: {};
}

export interface IMeshModule<V extends MeshType = MeshType>
  extends BaseModule<"Mesh"> {
  variation: V;

  materialId?: string;
  props: ComponentProps<typeof MESH_COMPONENTS[V]>;
}

export interface IColliderModule<V extends ColliderType = ColliderType>
  extends BaseModule<"Collider"> {
  variation: V;

  props: ComponentProps<typeof COLLIDER_COMPONENTS[V]>;
}

export type IModule = IMeshModule | IColliderModule;

//entity
export type Entity = {
  id: string;
  name?: string;

  transform: Transform;
  modules: IModule[];

  parentId: string | null;
  children: Entity[];
};

//material
export type Material = {
  id: string;
  name: string;

  color: string;
  emissive: string;
  opacity: number;
  roughness: number;
  metalness: number;
  flatShading: boolean;
  textureId?: string;
};

export type Materials = {
  [key: string]: Material;
};

//asset
export type AssetType = "image" | "model";

export type Asset = {
  type: AssetType;
  uri: string;
  data?: string;
};

export type Assets = {
  [key: string]: Asset;
};

//scene
export type Scene = {
  tree: Entity;
  materials: Materials;
  assets: Assets;
};
