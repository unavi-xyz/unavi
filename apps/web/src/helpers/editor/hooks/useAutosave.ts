import { useEffect } from "react";
import { useAtom } from "jotai";

import { getLocalWorld, setLocalWorldScene } from "../../localWorlds/db";
import { sceneAtom, worldIdAtom } from "../state";

export default function useAutosave() {
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
      setLocalWorldScene(worldId, scene);
    }, 10000);

    return () => {
      clearInterval(interval);
    };
  }, [worldId, scene]);
}
