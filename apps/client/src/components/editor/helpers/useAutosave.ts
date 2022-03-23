import { useEffect } from "react";

import { sceneManager, useStore } from "./store";
import { mergeLocalScene } from "../../scene/localScenes/db";
import { useLocalScene } from "../../scene/localScenes/useLocalScene";

export function useAutosave() {
  const sceneId = useStore((state) => state.sceneId);

  const localScene = useLocalScene(sceneId);

  useEffect(() => {
    //initial load
    if (localScene?.scene) {
      sceneManager.scene = localScene.scene;
    } else {
      sceneManager.scene = { assets: {}, instances: {}, materials: {} };
    }
  }, [localScene]);

  useEffect(() => {
    if (!sceneId) return;

    //save on an interval
    function save() {
      sceneManager.pruneAssets();
      const scene = useStore.getState().scene;
      mergeLocalScene(sceneId, { scene });
    }

    const interval = setInterval(save, 5000);
    return () => {
      clearInterval(interval);
    };
  }, [sceneId]);
}
