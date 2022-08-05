import { useEffect, useState } from "react";

import { useStudioStore } from "../store";
import { addObjectAsSibling, findObject, removeObjectFromScene } from "../utils/scene";

export function useStudioHotkeys() {
  const [copiedId, setCopiedId] = useState<string>();

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case "Delete":
          const selectedId = useStudioStore.getState().selectedId;
          if (selectedId) {
            const object = findObject(selectedId);
            if (!object) return;

            removeObjectFromScene(object);
            useStudioStore.setState({ selectedId: null });
          }
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
        case "c":
          // Copy
          if (e.ctrlKey) {
            const selectedId = useStudioStore.getState().selectedId;
            if (selectedId) setCopiedId(selectedId);
          }
          break;
        case "v":
          // Pase
          if (e.ctrlKey) {
            if (!copiedId) return;

            const copiedObject = findObject(copiedId);
            if (!copiedObject) return;

            const clone = copiedObject.clone();
            addObjectAsSibling(clone, copiedObject, "below");
          }
          break;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [copiedId]);
}
