import { useEffect } from "react";

import { useSave } from "./useSave";

export function useAutosave() {
  const { save } = useSave();

  useEffect(() => {
    // Save every 30 seconds
    const interval = setInterval(save, 30000);

    return () => clearInterval(interval);
  }, [save]);
}
