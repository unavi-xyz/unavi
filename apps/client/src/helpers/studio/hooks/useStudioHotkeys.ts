import { nanoid } from "nanoid";
import { useEffect, useState } from "react";

import { findEntityById } from "@wired-xr/scene";

import { useStudioStore } from "../store";

export function useHotkeys() {
  const [copiedId, setCopiedId] = useState<string>();

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
        case "c":
          if (e.ctrlKey) {
            //copy
            const selectedId = useStudioStore.getState().selectedId;
            const scene = useStudioStore.getState().scene;
            if (selectedId) {
              setCopiedId(selectedId);
            }
          }
          break;
        case "v":
          if (e.ctrlKey) {
            //paste
            if (!copiedId) return;
            const scene = useStudioStore.getState().scene;
            const copiedEntity = findEntityById(scene.tree, copiedId);
            if (!copiedEntity) return;

            const newEntity = { ...copiedEntity, id: nanoid() };
            useStudioStore.getState().addEntity(newEntity);
            useStudioStore.setState({ selectedId: newEntity.id });
          }
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
  }, [copiedId]);
}
