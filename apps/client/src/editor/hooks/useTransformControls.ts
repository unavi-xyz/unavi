import { useEffect } from "react";

import { useEditorStore } from "../store";

export function useTransformControls() {
  const engine = useEditorStore((state) => state.engine);
  const selectedId = useEditorStore((state) => state.selectedId);
  const tool = useEditorStore((state) => state.tool);

  // Update selected object when user clicks on an object
  useEffect(() => {
    if (!engine) return;

    engine.modules.render.addEventListener("clicked_node", (e) => {
      const nodeId = e.data.nodeId;
      useEditorStore.setState({ selectedId: nodeId });
    });
  }, [engine]);

  // Attach transform controls to the selected object
  useEffect(() => {
    if (!engine) return;

    engine.modules.render.toRenderThread({
      subject: "set_transform_controls_target",
      data: { nodeId: selectedId },
    });
  }, [engine, selectedId]);

  // Set mode
  useEffect(() => {
    if (!engine) return;

    engine.modules.render.toRenderThread({
      subject: "set_transform_controls_mode",
      data: tool,
    });
  }, [engine, tool]);
}
