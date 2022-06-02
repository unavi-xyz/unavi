import produce from "immer";
import { useEffect, useState } from "react";

import { Scene } from "@wired-xr/scene";

import { loadFromIpfs } from "../../ipfs/fetch";

export function useLoadAssets(sceneString: string) {
  const [loadedScene, setLoadedScene] = useState<Scene>();

  useEffect(() => {
    if (!sceneString) return;
    const scene: Scene = JSON.parse(sceneString);

    async function loadScene() {
      try {
        const newScene = await produce(scene, async (draft) => {
          await Promise.all(
            Object.entries(draft.assets).map(async ([key, asset]) => {
              if (asset.uri.startsWith("ipfs://")) {
                const hash = asset.uri.replace("ipfs://", "");
                const data = await loadFromIpfs(hash);

                if (asset.type === "image" || asset.type === "model") {
                  draft.assets[key].data = data;
                } else if (asset.type === "material") {
                  const res = await fetch(data);
                  const json = await res.json();
                  draft.assets[key].data = json;
                }
              }
            })
          );
        });

        setLoadedScene(newScene);
      } catch (error) {
        console.error(error);
        setLoadedScene(undefined);
      }
    }

    loadScene();
  }, [sceneString]);

  return loadedScene;
}
