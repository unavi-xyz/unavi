import { useEffect, useRef } from "react";
import { ASSETS, ASSET_NAMES, EditorObject, PARAM_NAMES } from "3d";

import { useScene } from "../state/useScene";
import { useStore, TOOLS } from "../state/useStore";

export function useHotkeys() {
  const selected = useStore((state) => state.selected);
  const setSelected = useStore((state) => state.setSelected);
  const setTool = useStore((state) => state.setTool);
  const deleteObject = useScene((state) => state.deleteObject);
  const addObject = useScene((state) => state.addObject);
  const scene = useScene((state) => state.scene);

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
            const count = Object.values(scene).filter(
              (item) => item.instance.type === copied.current.instance.type
            ).length;
            const limit = ASSETS[copied.current.instance.type].limit;
            if (count >= limit && limit >= 0) return;

            const obj = copied.current.clone();
            obj.instance.params[PARAM_NAMES.position][0] += 0.2;
            obj.instance.params[PARAM_NAMES.position][1] += 0.2;
            obj.instance.params[PARAM_NAMES.position][2] += 0.2;

            addObject(obj);
            setSelected(obj);
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
  }, [addObject, deleteObject, scene, selected, setSelected, setTool]);
}
