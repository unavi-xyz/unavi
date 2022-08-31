import { useEffect, useState } from "react";

import { removeEntity } from "../actions/RemoveEntityAction";
import { useStudioStore } from "../store";

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
            removeEntity(selectedId);
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
          // // Copy
          // if (e.ctrlKey) {
          //   const selectedId = useStudioStore.getState().selectedId;
          //   if (selectedId) setCopiedId(selectedId);
          // }
          break;
        case "v":
          // // Paste
          // if (e.ctrlKey) {
          //   if (!copiedId) return;

          //   const object = getObject(copiedId);
          //   if (!object) return;

          //   const clone = object.clone();
          //   addItemAsSibling(clone, object, "above");
          // }
          break;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [copiedId]);
}
