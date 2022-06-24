import { useEffect, useState } from "react";

import { readProject } from "../filesystem";
import { useStudioStore } from "../store";
import { Project } from "../types";

export function useProject() {
  const root = useStudioStore((state) => state.rootHandle);

  const [project, setProject] = useState<Project>();

  useEffect(() => {
    if (!root) {
      setProject(undefined);
      return;
    }

    readProject()
      .then((data) => {
        setProject(data);
      })
      .catch((err) => {
        console.error(err);
        setProject(undefined);
      });
  }, [root]);

  return project;
}
