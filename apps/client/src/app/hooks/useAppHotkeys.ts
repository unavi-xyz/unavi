import { useEffect } from "react";

import { useAppStore } from "../store";

export function useAppHotkeys() {
  const chatInputRef = useAppStore((state) => state.chatInputRef);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case "t":
        case "T":
          if (!chatInputRef.current) break;
          chatInputRef.current.focus();
          e.preventDefault();
          document.exitPointerLock();
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [chatInputRef]);
}
