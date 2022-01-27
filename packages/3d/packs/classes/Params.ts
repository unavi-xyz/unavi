import { Triplet } from "@react-three/cannon";

import { ASSET_NAMES } from "..";

export class Params {
  type: ASSET_NAMES = ASSET_NAMES.Box;
  name: string = "Object";

  position: Triplet = [0, 0, 0];
  rotation: Triplet = [0, 0, 0];
  scale: Triplet = [1, 1, 1];

  radius: number = 0.5;

  color: string = "#ffffff";
  opacity: number = 1;

  physEnabled: boolean = false;
  physType: "Dynamic" | "Static" = "Static";
  mass: number = 1;
}
