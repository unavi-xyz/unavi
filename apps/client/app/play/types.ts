import { WorldId } from "@/src/utils/parseWorldId";

export type WorldUriId = WorldId | { type: "uri"; value: string };

export enum PlayMode {
  Play = "play",
  Edit = "edit",
}

export enum RightPanelPage {
  World = "world",
}

export enum LeftPanelPage {
  Add = "add",
  Scene = "scene",
}
