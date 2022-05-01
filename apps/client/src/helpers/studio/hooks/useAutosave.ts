import { useEffect } from "react";

import { useStudioStore } from "../store";
import {
  getLocalSpace,
  updateLocalSpace,
} from "../../indexedDB/localSpaces/helpers";

export function useAutosave() {
  const id = useStudioStore((state) => state.id);

  // load initial space
  useEffect(() => {
    if (!id) return;

    getLocalSpace(id)
      .then((localSpace) => {
        if (!localSpace) return;

        useStudioStore.setState({
          name: localSpace.name,
          scene: localSpace.scene,
        });
      })
      .catch(console.error);
  }, [id]);

  // autosave on an interval
  useEffect(() => {
    const interval = setInterval(() => {
      const scene = useStudioStore.getState().scene;
      updateLocalSpace(id, { scene });
    }, 2000);

    return () => clearInterval(interval);
  }, [id]);
}
