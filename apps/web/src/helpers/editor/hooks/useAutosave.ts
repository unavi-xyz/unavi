import { useEffect } from "react";
import { useAtom } from "jotai";

import { getLocalWorld, mergeLocalWorld } from "../../localWorlds/db";
import { worldIdAtom } from "../state";
import { useStore } from "../store";

export function useAutosave() {
  const setScene = useStore((state) => state.setScene);
  const setSelected = useStore((state) => state.setSelected);

  const [worldId, setWorldId] = useAtom(worldIdAtom);

  useEffect(() => {
    //initial load
    if (!worldId) return;

    getLocalWorld(worldId).then((res) => {
      setScene(res?.scene);
    });
  }, [setScene, setSelected, setWorldId, worldId]);

  useEffect(() => {
    if (!worldId) return;

    //save on an interval
    const interval = setInterval(() => {
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

      const savedScene = { ...scene, textures: newTextures };

      mergeLocalWorld(worldId, { scene: savedScene });
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [worldId]);
}
