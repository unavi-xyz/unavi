import { useEffect } from "react";

import { useStudioStore } from "../store";

export function useTransformControls() {
  const engine = useStudioStore((state) => state.engine);
  const selectedId = useStudioStore((state) => state.selectedId);
  const tool = useStudioStore((state) => state.tool);

  // Update selected object when user clicks on an object
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.onObjectClick = (eid) => useStudioStore.setState({ selectedId: eid });
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
