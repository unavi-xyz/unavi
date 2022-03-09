import { nanoid } from "nanoid";
import { AssetName, Instance, Scene } from "3d";

import { ASSETS } from "./types";

export function newInstance(asset: AssetName, scene: Scene) {
  const id = nanoid();
  const instance: Instance = { id, asset, params: { ...ASSETS.Box.params } };
  const newScene: Scene = { ...scene, [id]: instance };
  return newScene;
}
