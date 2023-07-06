import { WorldId } from "@/src/utils/parseWorldId";

export type WorldUriId = WorldId | { type: "uri"; value: string };

export enum PlayMode {
  Play = "play",
  Build = "build",
}

export enum Tool {
  Translate = "translate",
  Rotate = "rotate",
  Scale = "scale",
}

export enum RightPanelPage {
  World = "world",
}

export enum LeftPanelPage {
  Add = "add",
}
