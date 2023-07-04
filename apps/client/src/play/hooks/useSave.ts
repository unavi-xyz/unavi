import { useCallback } from "react";

import { usePlayStore } from "@/app/play/store";

export function useSave() {
  const save = useCallback(async () => {
    const metadata = usePlayStore.getState().metadata;
  }, []);

  return save;
}
