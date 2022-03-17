import { useEffect } from "react";
import { useAtom } from "jotai";
import { Scene } from "3d";

import { worldIdAtom } from "./state";
import { useStore } from "./store";
import { mergeLocalScene } from "../scene/localScenes/db";
import { useLocalScene } from "../scene/localScenes/useLocalScene";

const defaultScene: Scene = { instances: {}, models: {}, textures: {} };

export function useAutosave() {
  const [id] = useAtom(worldIdAtom);

  const localScene = useLocalScene(id);

  useEffect(() => {
    //initial load
    useStore.getState().setScene(localScene?.scene ?? defaultScene);
  }, [localScene]);

  useEffect(() => {
    if (!id) return;

    function save() {
      const scene = useStore.getState().scene;

      //prune unused textures
      const usedTextures = Object.values(scene.instances).map(
        (instance) => instance.params?.texture
      );
      const filteredTextures = Object.keys(scene.textures).filter((id) =>
        usedTextures.includes(id)
      );

      const newTextures = {};
      filteredTextures.forEach((id) => {
        newTextures[id] = scene.textures[id];
      });

      //prune unused models
      const usedModels = Object.values(scene.instances).map(
        (instance) => instance.params?.model
      );
      const filteredModels = Object.keys(scene.models).filter((id) =>
        usedModels.includes(id)
      );

      const newModels = {};
      filteredModels.forEach((id) => {
        newModels[id] = scene.models[id];
      });

      //save scene
      const newScene = { ...scene, textures: newTextures, models: newModels };
      mergeLocalScene(id, { scene: newScene });
    }

    //save on an interval
    const interval = setInterval(save, 5000);
    return () => {
      clearInterval(interval);
    };
  }, [id]);
}
