import { useEffect } from "react";

import { updateEntity } from "../actions/UpdateEntityAction";
import { useEditorStore } from "../store";

export function useTransformControls() {
  const engine = useEditorStore((state) => state.engine);
  const selectedId = useEditorStore((state) => state.selectedId);
  const tool = useEditorStore((state) => state.tool);

  // Update selected object when user clicks on an object
  useEffect(() => {
    if (!engine) return;

    engine.renderThread.onObjectClick = (id) => {
      useEditorStore.setState({ selectedId: id });
    };
  }, [engine]);

  // Attach transform controls to the selected object
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.setTransformTarget(selectedId);
  }, [engine, selectedId]);

  // Set mode
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.setTransformMode(tool);
  }, [engine, tool]);
}
