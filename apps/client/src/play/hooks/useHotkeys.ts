import { useEffect } from "react";

import { usePlayStore } from "../store";

export function useHotkeys() {
  useEffect(() => {
    async function handleKeyDown(e: KeyboardEvent) {
      const isPointerLocked = document.pointerLockElement !== null;
      if (!isPointerLocked) {
        e.stopImmediatePropagation();
        return;
      }

      switch (e.key) {
        case "Enter":
        case "T":
        case "t": {
          e.preventDefault();
          usePlayStore.setState({ chatBoxFocused: true });
          break;
        }
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);
}
