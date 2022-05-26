import { useEffect } from "react";

import { writeScene } from "../filesystem";
import { useStudioStore } from "../store";
import { useProject } from "./useProject";

export function useAutosave() {
  const scene = useStudioStore((state) => state.scene);
  const project = useProject();

  //load initial space
  useEffect(() => {
    if (!project) return;
    useStudioStore.setState({ scene: project.scene });
  }, [project]);

  //autosave on scene change
  useEffect(() => {
    if (!project) return;

    //this is a hack to prevent the autosave from triggering on the initial project load
    //debounce autosave
    const timeout = setTimeout(() => {
      writeScene(scene).catch((err) => {
        console.error(err);
      });
    }, 250);

    return () => {
      clearTimeout(timeout);
    };
  }, [project, scene]);
}
