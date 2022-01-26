import { ASSET_NAMES } from "..";

export enum PROPERTIES {
  position,
  rotation,
  scale,
  color,
  opacity,
}

//each Asset defines a type of SceneObject
export class Asset {
  name: ASSET_NAMES;
  properties: PROPERTIES[];

  constructor(name: ASSET_NAMES, properties: PROPERTIES[]) {
    this.name = name;
    this.properties = properties;
  }
}
