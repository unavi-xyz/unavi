import { MutableRefObject } from "react";
import { Group } from "three";
import { Asset, Instance, ASSETS } from "3d";

export enum Tool {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export type Selected = {
  instance: Instance;
  ref: MutableRefObject<Group>;
};

export const PACKS: { [key: string]: Asset[] } = {
  Basic: [ASSETS.Box, ASSETS.Sphere],
  Game: [],
};
