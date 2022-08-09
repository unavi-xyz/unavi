import { useEffect } from "react";

import { useStudioStore } from "../store";

export function useOrbitControls() {
  const engine = useStudioStore((state) => state.engine);
  const usingTransform = useStudioStore((state) => state.usingTransform);

  // Create orbit controls
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.createOrbitControls();
    return () => {
      engine.renderThread.destroyOrbitControls();
    };
  }, [engine]);

  // Enable and disable orbit controls
  useEffect(() => {
    if (!engine) return;
    engine.renderThread.setOrbitControlsEnabled(!usingTransform);
  }, [engine, usingTransform]);
}
