import { useEffect } from "react";
import { useAtom } from "jotai";

import { getLocalWorld, mergeLocalWorld } from "../../localWorlds/db";
import { sceneAtom, worldIdAtom } from "../state";

export function useAutosave() {
  const [worldId] = useAtom(worldIdAtom);
  const [scene, setScene] = useAtom(sceneAtom);

  useEffect(() => {
    //initial load
    if (!worldId) return;
    getLocalWorld(worldId).then((res) => {
      setScene(res?.scene);
    });
  }, [setScene, worldId]);

  useEffect(() => {
    //save on an interval
    const interval = setInterval(() => {
      mergeLocalWorld(worldId, { scene });
    }, 10000);

    return () => {
      clearInterval(interval);
    };
  }, [worldId, scene]);
}
