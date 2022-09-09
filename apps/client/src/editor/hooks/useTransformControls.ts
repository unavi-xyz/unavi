import { useEffect } from "react";

import { useEditorStore } from "../store";

export function useTransformControls() {
  const engine = useEditorStore((state) => state.engine);
  const selectedId = useEditorStore((state) => state.selectedId);
  const tool = useEditorStore((state) => state.tool);

  // Update selected object when user clicks on an object
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.onObjectClick = (id) =>
      useEditorStore.setState({ selectedId: id });
    engine.renderThread.onSetTransform = (id, position, rotation, scale) => {
      const { scene } = useEditorStore.getState();
      const entity = scene.entities[id];
      entity.position = [position[0], position[1], position[2]];
      entity.rotation = [rotation[0], rotation[1], rotation[2]];
      entity.scale = [scale[0], scale[1], scale[2]];
      useEditorStore.setState({ scene });
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
