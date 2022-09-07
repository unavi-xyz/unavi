import { useEffect } from "react";

import { useStudioStore } from "../store";

export function useTransformControls() {
  const engine = useStudioStore((state) => state.engine);
  const selectedId = useStudioStore((state) => state.selectedId);
  const tool = useStudioStore((state) => state.tool);

  // Update selected object when user clicks on an object
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.onObjectClick = (id) =>
      useStudioStore.setState({ selectedId: id });
    engine.renderThread.onSetTransform = (id, position, rotation, scale) => {
      const { tree } = useStudioStore.getState();
      const entity = tree[id];
      entity.position = [position[0], position[1], position[2]];
      entity.rotation = [rotation[0], rotation[1], rotation[2]];
      entity.scale = [scale[0], scale[1], scale[2]];
      useStudioStore.setState({ tree });
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
