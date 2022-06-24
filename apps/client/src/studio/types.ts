import { IScene } from "@wired-xr/engine";

export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  Entity = "Entity",
}

export type Project = {
  name: string;
  description: string;

  scene: IScene;
};
