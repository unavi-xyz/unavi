import { useEffect } from "react";
import { useStore } from "../store";

export function useAppHotkeys() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case "t":
          const inputRef = useStore.getState().chatInputRef;
          inputRef.current.focus();
          e.preventDefault();
          document.exitPointerLock();
        default:
          break;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);
}
