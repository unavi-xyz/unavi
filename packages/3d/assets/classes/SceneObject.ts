import { nanoid } from "nanoid";

import { ASSETS, ASSET_NAMES } from "../assets";

export class SceneObject<T extends ASSET_NAMES> {
  id: string;
  type: ASSET_NAMES;
  params: typeof ASSETS[T]["params"];
  hidden: boolean;

  constructor(type: ASSET_NAMES, params: typeof ASSETS[T]["params"]) {
    this.id = nanoid();
    this.type = type;
    this.params = params;
    this.hidden = ASSETS[type].hidden;
  }
}
