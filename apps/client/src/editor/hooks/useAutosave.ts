import { useEffect } from "react";

import { useEditorStore } from "../store";
import { useSave } from "./useSave";

export function useAutosave() {
  const { save } = useSave();

  useEffect(() => {
    // Attempt auto save every 30 seconds
    const interval = setInterval(async () => {
      // Only save if there have been changes
      const { changesToSave } = useEditorStore.getState();
      if (!changesToSave) return;

      await save();
    }, 30000);

    return () => clearInterval(interval);
  }, [save]);
}
