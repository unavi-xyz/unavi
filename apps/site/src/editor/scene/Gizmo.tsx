import { useEffect, useState } from "react";
import { TransformControls } from "@react-three/drei";
import { PARAM_NAMES } from "3d";

import { TOOLS, useStore } from "../hooks/useStore";

export default function Gizmo() {
  const selected = useStore((state) => state.selected);
  const setSelected = useStore((state) => state.setSelected);
  const tool = useStore((state) => state.tool);
  const setUsingGizmo = useStore((state) => state.setUsingGizmo);

  const [enabled, setEnabled] = useState(false);

  useEffect(() => {
    return () => setSelected(undefined);
  }, [setSelected]);

  useEffect(() => {
    const params = selected?.instance?.params;

    if (!params) {
      setEnabled(false);
      return;
    }

    switch (tool) {
      case TOOLS.translate:
        setEnabled(PARAM_NAMES.position in params);
        break;
      case TOOLS.rotate:
        setEnabled(PARAM_NAMES.rotation in params);
        break;
      case TOOLS.scale:
        setEnabled(PARAM_NAMES.scale in params);
        break;
      default:
        setEnabled(false);
        break;
    }
  }, [selected?.instance?.params, tool]);

  function handleMouseDown() {
    setUsingGizmo(true);
  }

  function handleMouseUp() {
    selected.save();
    setUsingGizmo(false);
  }

  return (
    <TransformControls
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      object={selected?.ref}
      enabled={enabled}
      showX={Boolean(selected)}
      showY={Boolean(selected)}
      showZ={Boolean(selected)}
      size={0.7}
      mode={tool}
    />
  );
}
