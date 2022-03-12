import { useEffect, useState } from "react";
import { useAtom } from "jotai";
import { TransformControls } from "@react-three/drei";

import {
  saveSelectedAtom,
  toolAtom,
  usingGizmoAtom,
} from "../../../helpers/editor/state";
import { Tool } from "../../../helpers/editor/types";

export default function Gizmo() {
  const [tool] = useAtom(toolAtom);
  const [selected, saveSelected] = useAtom(saveSelectedAtom);
  const [, setUsingGizmo] = useAtom(usingGizmoAtom);

  const [enabled, setEnabled] = useState(false);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (selected) {
      if (tool === Tool.scale) {
        const hasScale = Boolean(selected.instance.params?.scale);
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
