import { useEffect } from "react";

import {
  getLocalSpace,
  updateLocalSpace,
} from "../../indexedDB/LocalSpace/helpers";
import { useStudioStore } from "../store";

export function useAutosave() {
  const id = useStudioStore((state) => state.id);
  const scene = useStudioStore((state) => state.scene);

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

    return () => {
      useStudioStore.setState({
        name: undefined,
      });
    };
  }, [id]);

  // autosave on an interval
  useEffect(() => {
    if (!id) return;
    updateLocalSpace(id, { scene });
  }, [id, scene]);
}
