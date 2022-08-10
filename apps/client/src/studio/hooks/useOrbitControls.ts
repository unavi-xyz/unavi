import { useEffect, useState } from "react";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";

import { useStudioStore } from "../store";

export function useOrbitControls() {
  const engine = useStudioStore((state) => state.engine);
  const usingTransform = useStudioStore((state) => state.usingTransformControls);

  const [controls, setControls] = useState<OrbitControls>();

  // Create orbit controls
  useEffect(() => {
    if (!engine) return;
    const { camera, renderer } = engine.renderManager;

    const orbitControls = new OrbitControls(camera, renderer.domElement);
    setControls(orbitControls);

    return () => {
      orbitControls.dispose();
    };
  }, [engine]);

  // Enable and disable orbit controls
  useEffect(() => {
    if (!engine || !controls) return;
    controls.enabled = !usingTransform;
  }, [engine, controls, usingTransform]);
}
