import { useEffect, useState } from "react";
import { TransformControls } from "@react-three/drei";

import { Tool } from "../helpers/types";
import { editorManager, useStore } from "../helpers/store";

export default function Gizmo() {
  const selected = useStore((state) => state.selected);
  const tool = useStore((state) => state.tool);

  const [enabled, setEnabled] = useState(false);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!selected) {
      setEnabled(false);
      setVisible(false);
      return;
    }

    setVisible(true);

    if (tool === Tool.scale) {
      const instance = useStore.getState().scene.instances[selected.id];
      const hasScale = "scale" in instance.properties;
      setEnabled(hasScale);
    } else {
      setEnabled(true);
    }
  }, [selected, tool]);

  function handleMouseDown() {
    editorManager.setUsingGizmo(true);
  }

  function handleMouseUp() {
    editorManager.saveSelected();
    editorManager.setUsingGizmo(false);
  }

  return (
    <TransformControls
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      object={selected?.ref?.current}
      showX={visible}
      showY={visible}
      showZ={visible}
      enabled={enabled}
      mode={tool}
    />
  );
}
