import { useEffect, useState } from "react";
import { useAtom, useSetAtom } from "jotai";
import { TransformControls } from "@react-three/drei";

import { toolAtom, usingGizmoAtom } from "../../../helpers/editor/state";
import { Tool } from "../../../helpers/editor/types";
import { useStore } from "../../../helpers/editor/store";

export default function Gizmo() {
  const selected = useStore((state) => state.selected);

  const setUsingGizmo = useSetAtom(usingGizmoAtom);
  const [tool] = useAtom(toolAtom);

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
      const params = useStore.getState().scene.instances[selected.id].params;
      const hasScale = params.hasOwnProperty("scale");
      setEnabled(hasScale);
    } else {
      setEnabled(true);
    }
  }, [selected, tool]);

  function handleMouseDown() {
    setUsingGizmo(true);
  }

  function handleMouseUp() {
    useStore.getState().saveSelected();
    setUsingGizmo(false);
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
