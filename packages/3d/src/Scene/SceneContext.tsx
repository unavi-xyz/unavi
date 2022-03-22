import { createContext } from "react";
import { Scene } from "./types";

interface ISceneContext {
  assets: Scene["assets"];
  materials: Scene["materials"];
}

const defaultValue: ISceneContext = {
  assets: {},
  materials: {},
};

export const SceneContext = createContext(defaultValue);
