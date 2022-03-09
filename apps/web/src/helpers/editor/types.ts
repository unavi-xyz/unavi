import { MutableRefObject } from "react";
import { Group } from "three";
import { Asset, AssetName, Instance, boxDefaultParams } from "3d";

export enum Tool {
  translate = "translate",
  rotate = "rotate",
  scale = "scale",
}

export type Selected = {
  instance: Instance;
  ref: MutableRefObject<Group>;
};

export const ASSETS = {
  Box: {
    name: AssetName.Box,
    params: boxDefaultParams,
  } as Asset,
};

export const PACKS: { [key: string]: Asset[] } = {
  Basic: [ASSETS.Box],
  Game: [],
};
