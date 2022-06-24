import { Triplet } from "@react-three/cannon";
import produce from "immer";
import { useEffect, useState } from "react";

import { IScene, traverseTree } from "@wired-xr/engine";

import { loadFromIpfs } from "../../ipfs/fetch";

export function useLoadAssets(sceneString: string) {
  const [loadedScene, setLoadedScene] = useState<IScene>();
  const [spawn, setSpawn] = useState<Triplet>();

  useEffect(() => {
    if (!sceneString) return;
    const scene: IScene = JSON.parse(sceneString);

    async function loadScene() {
      try {
        //load assets
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

        //set spawn
        const spawns: Triplet[] = [];
        traverseTree(newScene.tree, (entity) => {
          if (entity.type === "Spawn") spawns.push(entity.transform.position);
        });
        if (spawns.length > 0) setSpawn(spawns[0]);

        //set scene
        setLoadedScene(newScene);
      } catch (error) {
        console.error(error);
        setLoadedScene(undefined);
      }
    }

    loadScene();
  }, [sceneString]);

  return { loadedScene, spawn };
}
