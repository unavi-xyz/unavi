import produce from "immer";
import { useEffect } from "react";

import { traverseTree } from "@wired-xr/scene";

import { writeScene } from "../filesystem";
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
        await useStudioStore.getState().loadAsset(id);
      });

      //remove unnecessary data from scene
      const savedScene = produce(scene, (draft) => {
        //get all used materials
        const usedMaterials = new Set<string>();
        traverseTree(draft.tree, (entity) => {
          //@ts-ignore
          const materialId = entity.props?.materialId;
          if (materialId) usedMaterials.add(materialId);
        });

        Object.entries(draft.assets).forEach(([id, asset]) => {
          //remove unused materials
          if (asset.type === "material" && !usedMaterials.has(id)) {
            delete draft.assets[id];
            return;
          }

          //remove asset data
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
