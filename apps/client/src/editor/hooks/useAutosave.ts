import { useEffect } from "react";

import { useEditorStore } from "../store";
import { useSave } from "./useSave";

const AUTOSAVE_INTERVAL = 2 * 60 * 1000; // 2 minutes

export function useAutosave() {
  const { save } = useSave();

  useEffect(() => {
    // Auto save on an interval
    const interval = setInterval(() => {
      const { isPlaying } = useEditorStore.getState();
      if (isPlaying) return;
      save();
    }, AUTOSAVE_INTERVAL);

    return () => clearInterval(interval);
  }, [save]);
}
