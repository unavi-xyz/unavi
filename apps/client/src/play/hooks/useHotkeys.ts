import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";

export function useHotkeys() {
  useEffect(() => {
    async function handleKeyDown(e: KeyboardEvent) {
      const chatBoxFocused = usePlayStore.getState().chatBoxFocused;
      if (chatBoxFocused) {
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
