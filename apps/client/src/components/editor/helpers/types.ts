import { MutableRefObject } from "react";
import { Group } from "three";
import { SceneObjectType } from "3d";

export enum Tool {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export type Selected = {
  id: string;
  ref: MutableRefObject<Group>;
};

export const PACKS: { [key: string]: SceneObjectType[] } = {
  Basic: ["Box", "Sphere"],
};
