import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { PlayMode } from "@/app/play/types";

export function useHotkeys() {
  useEffect(() => {
    async function handleKeyDown(e: KeyboardEvent) {
      const { chatBoxFocused, mode } = usePlayStore.getState();

      if (mode !== PlayMode.Play) return;

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
