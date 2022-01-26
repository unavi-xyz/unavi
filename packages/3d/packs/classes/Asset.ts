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
  limit: number;

  constructor(name: ASSET_NAMES, properties: PROPERTIES[], limit = -1) {
    this.name = name;
    this.properties = properties;
    this.limit = limit;
  }
}
