import { useEffect, useState } from "react";
import { useAtom, useSetAtom } from "jotai";
import { TransformControls } from "@react-three/drei";

import { toolAtom, usingGizmoAtom } from "../../../helpers/editor/state";
import { Tool } from "../../../helpers/editor/types";
import { useStore } from "../../../helpers/editor/store";

export default function Gizmo() {
  const selected = useStore((state) => state.selected);
  const saveSelected = useStore((state) => state.saveSelected);

  const [tool] = useAtom(toolAtom);
  const setUsingGizmo = useSetAtom(usingGizmoAtom);

  const [enabled, setEnabled] = useState(false);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (selected) {
      const params = useStore.getState().scene.instances[selected.id].params;

      if (tool === Tool.scale) {
        const hasScale = Boolean(params?.scale);
        setEnabled(hasScale);
        setVisible(true);
      } else {
        setEnabled(true);
        setVisible(true);
      }
    } else {
      setEnabled(false);
      setVisible(false);
    }
  }, [selected, tool]);

  return (
    <TransformControls
      onMouseDown={() => {
        setUsingGizmo(true);
      }}
      onMouseUp={() => {
        saveSelected();
        setUsingGizmo(false);
      }}
      object={selected?.ref?.current}
      showX={visible}
      showY={visible}
      showZ={visible}
      enabled={enabled}
      mode={tool}
    />
  );
}
