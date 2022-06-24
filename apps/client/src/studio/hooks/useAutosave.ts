import produce from "immer";
import { useEffect } from "react";

import { traverseTree } from "@wired-xr/engine";

import { getFileByPath, writeScene } from "../filesystem";
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
  }, [project]);

  //autosave on scene change
  useEffect(() => {
    if (!project) return;

    //debounce autosave
    //this is a hack to prevent the autosave from triggering on the initial project load
    const timeout = setTimeout(() => {
      //load new assets
      Object.keys(project.scene.assets).forEach(async (id) => {
        try {
          await useStudioStore.getState().loadAsset(id);
        } catch (e) {
          console.error(e);
        }
      });

      //remove unnecessary data from scene
      const savedScene = produce(scene, (draft) => {
        //get used assets
        const usedAssets = new Set<string>();
        traverseTree(draft.tree, (entity) => {
          //@ts-ignore
          const materialId = entity.props?.materialId;
          //@ts-ignore
          const modelId = entity.props?.modelId;

          if (materialId) usedAssets.add(materialId);
          if (modelId) usedAssets.add(modelId);
        });

        //get used textures
        const usedTextures = new Set<string>();
        Object.entries(draft.assets).forEach(([id, asset]) => {
          if (asset.type === "material") {
            //@ts-ignore
            const textureId = asset.data?.textureId;
            if (textureId) usedTextures.add(textureId);
          }
        });

        //remove data that is not used
        Object.entries(draft.assets).forEach(([id, asset]) => {
          //remove asset if not used
          if (asset.type === "material" || asset.type === "model") {
            if (!usedAssets.has(id)) {
              delete draft.assets[id];
              return;
            }
          }

          if (asset.type === "image") {
            if (!usedTextures.has(id)) {
              delete draft.assets[id];
              return;
            }
          }

          //else just remove data
          if (asset.data) delete draft.assets[id].data;
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
