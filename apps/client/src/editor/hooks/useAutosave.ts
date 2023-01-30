import { useEffect } from "react";

import { useSave } from "./useSave";

const AUTOSAVE_INTERVAL = 3 * 60 * 1000; // 3 minutes

export function useAutosave() {
  const { save } = useSave();

  useEffect(() => {
    // Auto save on an interval
    const interval = setInterval(save, AUTOSAVE_INTERVAL);

    return () => clearInterval(interval);
  }, [save]);
}
