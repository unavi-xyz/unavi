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
    writeScene(scene).catch((err) => {
      console.error(err);
    });
  }, [project, scene]);
}
