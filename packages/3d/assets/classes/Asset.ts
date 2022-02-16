import { Triplet } from "@react-three/cannon";
import { ASSET_NAMES } from "../assets";

export enum PARAM_NAMES {
  type = "type",
  position = "position",
  rotation = "rotation",
  scale = "scale",
  radius = "radius",
  color = "color",
  opacity = "opacity",
  physEnabled = "physEnabled",
  physType = "physType",
  mass = "mass",
}

export type PARAMS = {
  [PARAM_NAMES.type]: ASSET_NAMES;
  [PARAM_NAMES.position]: Triplet;
  [PARAM_NAMES.rotation]: Triplet;
  [PARAM_NAMES.scale]: Triplet;
  [PARAM_NAMES.radius]: number;
  [PARAM_NAMES.color]: string;
  [PARAM_NAMES.opacity]: number;
  [PARAM_NAMES.physEnabled]: boolean;
  [PARAM_NAMES.physType]: "Static" | "Dynamic";
  [PARAM_NAMES.mass]: number;
};

export class Asset<T> {
  type: ASSET_NAMES;
  params: T;

  limit = -1;
  hidden = false;

  constructor(
    type: ASSET_NAMES,
    params: T,
    settings?: Partial<{ limit: number; hidden: boolean }>
  ) {
    this.type = type;
    this.params = params;

    if (settings?.limit) this.limit = settings.limit;
    if (settings?.hidden) this.hidden = settings.hidden;
  }
}
