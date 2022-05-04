import { useEffect } from "react";
import { useStudioStore } from "../store";

export function useHotkeys() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case "Delete":
          const selectedId = useStudioStore.getState().selectedId;
          if (selectedId) useStudioStore.getState().removeEntity(selectedId);
          useStudioStore.setState({ selectedId: undefined });
          break;
        case "w":
          useStudioStore.setState({ tool: "translate" });
          break;
        case "e":
          useStudioStore.setState({ tool: "rotate" });
          break;
        case "r":
          useStudioStore.setState({ tool: "scale" });
          break;
      }
    }

    function handleKeyUp(e: KeyboardEvent) {}

    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("keyup", handleKeyUp);
    };
  }, []);
}
