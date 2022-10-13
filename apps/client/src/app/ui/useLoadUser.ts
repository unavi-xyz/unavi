import { useEffect } from "react";

import { useLens } from "../../client/lens/hooks/useLens";
import { useAppStore } from "../store";
import { LocalStorageKey } from "./constants";

export function useLoadUser() {
  const engine = useAppStore((state) => state.engine);
  const { handle } = useLens();

  useEffect(() => {
    if (!engine) return;

    // Publish handle
    engine.setHandle(handle ?? null);
  }, [engine, handle]);

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
