import produce from "immer";
import { useEffect } from "react";

import { readFileByPath, writeScene } from "../filesystem";
import { useStudioStore } from "../store";
import { useProject } from "./useProject";

export function useAutosave() {
  const scene = useStudioStore((state) => state.scene);
  const project = useProject();

  //load initial space
  useEffect(() => {
    if (!project) return;

    //set scene
    useStudioStore.setState({ scene: project.scene });

    //fetch assets
    Object.entries(project.scene.assets).forEach(async ([id, asset]) => {
      const file = await readFileByPath(asset.uri);
      if (!file) return;

      const url = URL.createObjectURL(file);
      useStudioStore.getState().updateAsset(id, { data: url });
    });
  }, [project]);

  //autosave on scene change
  useEffect(() => {
    if (!project) return;

    //this is a hack to prevent the autosave from triggering on the initial project load
    //debounce autosave
    const timeout = setTimeout(() => {
      //remove unnecessary data from scene
      const savedScene = produce(scene, (draft) => {
        //get unused assets
        const usedAssetIds = new Set();
        Object.values(draft.materials).forEach((material) => {
          if (material.textureId) usedAssetIds.add(material.textureId);
        });

        //remove usnused assets
        Object.keys(draft.assets).forEach((id) => {
          if (!usedAssetIds.has(id)) delete draft.assets[id];
        });

        //remove asset data
        Object.entries(draft.assets).forEach(([id, asset]) => {
          delete asset.data;
        });
      });

      writeScene(savedScene).catch((err) => {
        console.error(err);
      });
    }, 250);

    return () => {
      clearTimeout(timeout);
    };
  }, [project, scene]);
}
