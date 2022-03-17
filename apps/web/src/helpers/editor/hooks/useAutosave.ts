import { useEffect } from "react";
import { useAtom } from "jotai";

import { worldIdAtom } from "../state";
import { useStore } from "../store";
import { mergeLocalScene } from "../../localScenes/db";
import { useLocalScene } from "../../localScenes/useLocalScene";

export function useAutosave() {
  const [id] = useAtom(worldIdAtom);

  const localScene = useLocalScene(id);

  useEffect(() => {
    //initial load
    useStore.getState().setScene(localScene?.scene);
  }, [localScene]);

  useEffect(() => {
    if (!id) return;

    function save() {
      const scene = useStore.getState().scene;

      //prune unused textures
      const usedTextures = Object.values(scene.instances).map(
        (instance) => instance.params?.texture
      );
      const filtered = Object.keys(scene.textures).filter((id) =>
        usedTextures.includes(id)
      );

      const newTextures = {};
      filtered.forEach((id) => {
        newTextures[id] = scene.textures[id];
      });

      const newScene = { ...scene, textures: newTextures };
      mergeLocalScene(id, { scene: newScene });
    }

    //save on an interval
    const interval = setInterval(save, 5000);
    return () => {
      clearInterval(interval);
    };
  }, [id]);
}
