import { useEffect, useRef } from "react";
import { editorManager, sceneManager, useStore } from "../store";

export function useEditorHotkeys() {
  const copied = useRef<string>();
  const holdingV = useRef(false);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const state = useStore.getState();
      const selected = state.selected;

      switch (e.key) {
        case "Delete":
          if (selected) {
            sceneManager.deleteInstance(selected?.id);
            editorManager.setSelected(undefined);
          }
          break;

        case "w":
          editorManager.setTool("translate");
          break;
        case "e":
          editorManager.setTool("rotate");
          break;
        case "r":
          editorManager.setTool("scale");
          break;

        case "c":
          if (e.ctrlKey) {
            copied.current = selected?.id;
          }
          break;
        case "v":
          if (e.ctrlKey && copied.current && !holdingV.current) {
            const instance = state.scene.instances[copied.current];

            const id = sceneManager.newInstance(instance.type);
            sceneManager.editInstance(id, instance.properties);
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
  }, []);
}
