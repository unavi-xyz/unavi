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

      mergeLocalWorld(worldId, { scene });
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [worldId]);
}
