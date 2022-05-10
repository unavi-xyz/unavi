import { useEffect } from "react";
import { updateLocalSpace } from "../../../helpers/indexedDB/LocalSpace/helpers";
import { useLocalSpace } from "../../../helpers/indexedDB/LocalSpace/hooks/useLocalScene";

import { sceneManager, useStore } from "../store";

export function useAutosave() {
  const sceneId = useStore((state) => state.sceneId);

  const localScene = useLocalSpace(sceneId);

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
      updateLocalSpace(sceneId, { scene });
    }

    const interval = setInterval(save, 5000);
    return () => {
      clearInterval(interval);
    };
  }, [sceneId]);
}
