import { useEffect, useState } from "react";

import { removeEntity } from "../actions/RemoveEntityAction";
import { useEditorStore } from "../store";

export function useEditorHotkeys() {
  const [copiedId, setCopiedId] = useState<string>();

  useEffect(() => {
    async function handleKeyDown(e: KeyboardEvent) {
      const engine = useEditorStore.getState().engine;
      if (!engine) return;

      switch (e.key) {
        case "Delete":
          const selectedId = useEditorStore.getState().selectedId;
          if (selectedId) {
            useEditorStore.setState({ selectedId: null });
            removeEntity(selectedId);
          }
          break;
        case "w":
          useEditorStore.setState({ tool: "translate" });
          break;
        case "e":
          useEditorStore.setState({ tool: "rotate" });
          break;
        case "r":
          useEditorStore.setState({ tool: "scale" });
          break;
        case "c":
          // // Copy
          // if (e.ctrlKey) {
          //   const selectedId = useEditorStore.getState().selectedId;
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
