import { Triplet } from "@react-three/cannon";
import { SceneObjects } from "./constants";

export type SceneObjectType = keyof typeof SceneObjects;

export type Instance<T extends SceneObjectType> = {
  id: string;
  type: T;
  properties: typeof SceneObjects[T]["properties"];
};

export type Properties = {
  position: Triplet;
  rotation: Triplet;
  scale: Triplet;
  radius: number;
  src: string | undefined;
  material: string | undefined;
};

export type Material = {
  id: string;
  type: "physical" | "toon";
  color: string;
  emissive: string;
  sheenColor: string;
  opacity: number;
  reflectivity: number;
  metalness: number;
  roughness: number;
  clearcoat: number;
  sheen: number;
  texture: string | undefined;
  flatShading: boolean;
  wireframe: boolean;
};

export type Scene = {
  instances: { [key: string]: Instance<SceneObjectType> };
  assets: { [key: string]: File };
  materials: { [key: string]: Material };
};

export type JsonScene = {
  instances: Scene["instances"];
  materials: Scene["materials"];
  assets: { [key: string]: string };
};
