import { useEffect } from "react";

import { useAppStore } from "../store";
import { LocalStorageKey } from "./constants";

export function useLoadUser() {
  const engine = useAppStore((state) => state.engine);

  useEffect(() => {
    if (!engine) return;

    const displayName = localStorage.getItem(LocalStorageKey.Name);
    const customAvatar = localStorage.getItem(LocalStorageKey.Avatar);

    useAppStore.setState({ displayName, customAvatar });

    // Publish info
    engine.setName(displayName);
    engine.setAvatar(customAvatar);
  }, [engine]);
}
