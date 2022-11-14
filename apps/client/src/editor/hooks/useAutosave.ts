import { useEffect } from "react";

import { useEditorStore } from "../store";
import { useSave } from "./useSave";

export function useAutosave() {
  const { save } = useSave();

  useEffect(() => {
    // Auto save on an interval
    const interval = setInterval(async () => {
      // Only save if there have been changes
      const { changesToSave } = useEditorStore.getState();
      if (!changesToSave) return;

      await save();
    }, 3 * 60 * 1000);

    return () => clearInterval(interval);
  }, [save]);
}
