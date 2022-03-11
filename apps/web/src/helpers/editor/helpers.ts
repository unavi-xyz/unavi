import { nanoid } from "nanoid";
import { AssetName, ASSETS, Instance, Scene } from "3d";

export function newInstance(name: AssetName, scene: Scene) {
  const id = nanoid();
  const params = ASSETS[name].params;
  const instance: Instance = { id, name, params: { ...params } };
  const newScene: Scene = { ...scene, [id]: instance };
  return newScene;
}
