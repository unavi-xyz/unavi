import { MutableRefObject } from "react";
import { Group } from "three";
import { SceneObjectType } from "3d";

export type Tool = "translate" | "rotate" | "scale";

export type Selected = {
  id: string;
  ref: MutableRefObject<Group>;
};

export const PACKS: { [key: string]: SceneObjectType[] } = {
  Basic: ["Box", "Sphere"],
};
