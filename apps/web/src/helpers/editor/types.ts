import { MutableRefObject } from "react";
import { Group } from "three";
import { Asset, ASSETS, Instance } from "3d";

export enum Tool {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export type Selected = {
  id: string;
  ref: MutableRefObject<Group>;
};

export const PACKS: { [key: string]: Asset[] } = {
  Basic: [ASSETS.Box, ASSETS.Sphere],
  Game: [],
};
