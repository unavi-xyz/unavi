import { useEffect } from "react";

import { useStudioStore } from "../store";
import { findItem } from "../utils/scene";

export function useTransformControls() {
  const engine = useStudioStore((state) => state.engine);
  const selectedId = useStudioStore((state) => state.selectedId);
  const tool = useStudioStore((state) => state.tool);

  // Create transform controls
  useEffect(() => {
    if (!engine) return;

    engine.renderThread.createTransformControls();

    engine.renderThread.onClickIntersection = (uuid) => {
      if (uuid) {
        const item = findItem(uuid, engine.tree, "threeUUID");

        if (item) {
          useStudioStore.setState({ selectedId: item.id });
          return;
        }
      }

      useStudioStore.setState({ selectedId: null });
    };

    return () => {
      engine.renderThread.destroyTransformControls();
    };
  }, [engine]);

  // Attach controls to selected object
  useEffect(() => {
    if (!engine) return;

    if (!selectedId) {
      engine.renderThread.detachTransformControls();
      return;
    }

    const item = findItem(selectedId, engine.tree);

    if (!item || !item.threeUUID) {
      engine.renderThread.detachTransformControls();
      return;
    }

    engine.renderThread.attachTransformControls(item.threeUUID);
  }, [engine, selectedId]);

  // Switch controls to tool
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.setTransformControlsMode(tool);
  }, [engine, tool]);
}
