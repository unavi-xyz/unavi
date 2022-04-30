import { useEffect } from "react";
import { mergeLocalSpace } from "../../../helpers/indexeddb/localSpaces/db";
import { useLocalSpace } from "../../../helpers/indexeddb/localSpaces/useLocalScene";

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
      mergeLocalSpace(sceneId, { scene });
    }

    const interval = setInterval(save, 5000);
    return () => {
      clearInterval(interval);
    };
  }, [sceneId]);
}
