import { useEffect, useRef } from "react";
import { useSetAtom } from "jotai";

import { toolAtom } from "../state";
import { useStore } from "../store";
import { Tool } from "../types";

export function useHotkeys() {
  const setTool = useSetAtom(toolAtom);

  const copied = useRef<string>();
  const holdingV = useRef(false);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const state = useStore.getState();
      const selected = state.selected;

      switch (e.key) {
        case "Delete":
          if (selected) {
            state.deleteInstance(selected?.id);
            state.setSelected(null);
          }
          break;

        case "w":
          setTool(Tool.translate);
          break;
        case "e":
          setTool(Tool.rotate);
          break;
        case "r":
          setTool(Tool.scale);
          break;

        case "c":
          if (e.ctrlKey) {
            copied.current = selected?.id;
          }
          break;
        case "v":
          if (e.ctrlKey && copied.current && !holdingV.current) {
            if (selected) state.saveSelected();

            const instance = state.scene.instances[copied.current];

            useStore
              .getState()
              .newInstance(instance.name, { ...instance.params });
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
  }, [setTool]);
}
