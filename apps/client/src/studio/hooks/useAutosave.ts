import { useEffect } from "react";

import { Project } from "@/src/server/helpers/fetchProject";

import { StudioMode } from "../components/Studio";
import { useSave } from "./useSave";

const AUTOSAVE_INTERVAL = 2 * 60 * 1000; // 2 minutes

export function useAutosave(project: Project, mode: StudioMode) {
  const { save } = useSave(project.id);

  useEffect(() => {
    if (mode !== "play") return;

    // Auto save on an interval
    const interval = setInterval(save, AUTOSAVE_INTERVAL);

    return () => clearInterval(interval);
  }, [mode, save]);
}
