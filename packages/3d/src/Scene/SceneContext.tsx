import { createContext } from "react";
import { Scene } from "./types";

interface ISceneContext {
  assets: Scene["assets"];
  materials: Scene["materials"];
  debug: boolean;
}

const defaultValue: ISceneContext = {
  assets: {},
  materials: {},
  debug: false,
};

export const SceneContext = createContext(defaultValue);
