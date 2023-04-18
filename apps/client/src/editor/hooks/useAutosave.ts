import { useEffect } from "react";

import { Project } from "@/src/server/helpers/fetchProject";

import { EditorMode } from "../components/Editor";
import { useSave } from "./useSave";

const AUTOSAVE_INTERVAL = 2 * 60 * 1000; // 2 minutes

export function useAutosave(project: Project, mode: EditorMode) {
  const { save } = useSave(project);

  useEffect(() => {
    if (mode !== "play") return;

    // Auto save on an interval
    const interval = setInterval(save, AUTOSAVE_INTERVAL);

    return () => clearInterval(interval);
  }, [mode, save]);
}
