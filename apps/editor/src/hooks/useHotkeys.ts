import { useEffect } from "react";

import { TOOLS, useScene } from "../state/useScene";
import { useStore } from "../state/useStore";

export function useHotkeys() {
  const selected = useStore((state) => state.selected);
  const setSelected = useStore((state) => state.setSelected);
  const setTool = useStore((state) => state.setTool);
  const deleteObject = useScene((state) => state.deleteObject);

  useEffect(() => {
    function handleKeypress(e: KeyboardEvent) {
      switch (e.key) {
        case "Delete":
          if (selected) {
            deleteObject(selected);
            setSelected(null, null);
          }
          break;

        case "w":
          setTool(TOOLS.translate);
          break;
        case "e":
          setTool(TOOLS.rotate);
          break;
        case "r":
          setTool(TOOLS.scale);
          break;

        default:
          break;
      }
    }

    document.addEventListener("keydown", handleKeypress);
    return () => {
      document.removeEventListener("keydown", handleKeypress);
    };
  }, [deleteObject, selected, setSelected, setTool]);
}
