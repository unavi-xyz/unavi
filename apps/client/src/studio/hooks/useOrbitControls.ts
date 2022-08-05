import { useEffect, useState } from "react";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";

import { useStudioStore } from "../store";

export function useOrbitControls() {
  const engine = useStudioStore((state) => state.engine);
  const usingTransform = useStudioStore((state) => state.usingTransform);

  const [controls, setControls] = useState<OrbitControls>();

  useEffect(() => {
    if (!engine) return;

    const canvas = engine.renderer.domElement;
    const orbitControls = new OrbitControls(engine.camera, canvas);
    setControls(orbitControls);

    return () => {
      orbitControls.dispose();
    };
  }, [engine]);

  useEffect(() => {
    if (!controls) return;
    controls.enabled = !usingTransform;
  }, [usingTransform, controls]);
}
