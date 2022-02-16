import { useEffect, useRef } from "react";
import { ASSET_NAMES, EditorObject } from "3d";

import { useStore, TOOLS } from "./useStore";

export function useHotkeys() {
  const selected = useStore((state) => state.selected);
  const setSelected = useStore((state) => state.setSelected);
  const setTool = useStore((state) => state.setTool);
  const deleteObject = useStore((state) => state.deleteObject);
  const addObject = useStore((state) => state.addObject);
  const objects = useStore((state) => state.objects);

  const copied = useRef<EditorObject<ASSET_NAMES>>();
  const holdingV = useRef(false);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case "Delete":
          if (selected) {
            deleteObject(selected);
            setSelected(null);
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

        case "c":
          if (e.ctrlKey) {
            copied.current = selected;
          }
          break;
        case "v":
          if (e.ctrlKey && copied.current && !holdingV.current) {
            const obj = copied.current.clone();

            const success = addObject(obj);
            if (success) setSelected(obj);
          }

          holdingV.current = true;
          break;

        default:
          break;
      }
    }

    function handleKeyUp(e: KeyboardEvent) {
      switch (e.key) {
        case "v":
          holdingV.current = false;
        default:
          break;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("keyup", handleKeyUp);
    };
  }, [addObject, deleteObject, objects, selected, setSelected, setTool]);
}
