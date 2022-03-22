import { createContext } from "react";

interface ISceneContext {
  assets: { [key: string]: File };
}

const defaultValue: ISceneContext = {
  assets: {},
};

export const SceneContext = createContext(defaultValue);
