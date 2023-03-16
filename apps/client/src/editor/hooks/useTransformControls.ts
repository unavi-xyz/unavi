import { useEffect } from "react";

import { useEditorStore } from "../store";

export function useTransformControls() {
  const engine = useEditorStore((state) => state.engine);
  const selectedId = useEditorStore((state) => state.selectedId);
  const tool = useEditorStore((state) => state.tool);

  // Update selected object when user clicks on an object
  useEffect(() => {
    if (!engine) return;

    engine.render.addEventListener("clicked_node", (e) => {
      const { isPlaying } = useEditorStore.getState();
      if (isPlaying) return;

      const nodeId = e.data.nodeId;
      useEditorStore.setState({ selectedId: nodeId });
    });
  }, [engine]);

  // Attach transform controls to the selected object
  useEffect(() => {
    if (!engine) return;

    const { isPlaying } = useEditorStore.getState();

    engine.render.send({
      subject: "set_transform_controls_target",
      data: { nodeId: selectedId, attach: !isPlaying },
    });
  }, [engine, selectedId]);

  // Set mode
  useEffect(() => {
    if (!engine) return;

    engine.render.send({
      subject: "set_transform_controls_mode",
      data: tool,
    });
  }, [engine, tool]);
}
