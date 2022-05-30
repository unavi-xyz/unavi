import { Scene } from "@wired-xr/scene";

export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  Entity = "Entity",
}

export type Project = {
  name: string;
  description: string;

  scene: Scene;
};
