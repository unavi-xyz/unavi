import { useEffect, useState } from "react";

import { useStudioStore } from "../store";
import { addItemAsSibling, cloneItem, findItem } from "../utils/scene";

export function useStudioHotkeys() {
  const [copiedId, setCopiedId] = useState<string>();

  useEffect(() => {
    async function handleKeyDown(e: KeyboardEvent) {
      const engine = useStudioStore.getState().engine;
      if (!engine) return;

      switch (e.key) {
        case "Delete":
          const selectedId = useStudioStore.getState().selectedId;
          if (selectedId) {
            const item = findItem(selectedId, engine.tree);
            if (!item) return;
            item.removeFromParent();
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

            const copiedItem = findItem(copiedId, engine.tree);
            if (!copiedItem) return;

            const clone = await cloneItem(copiedItem, engine);
            addItemAsSibling(clone, copiedItem, "above");
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
