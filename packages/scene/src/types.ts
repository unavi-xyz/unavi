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

//scene
export type Scene = {
  tree: Entity;
};
