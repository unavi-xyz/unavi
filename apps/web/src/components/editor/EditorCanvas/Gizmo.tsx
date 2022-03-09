import { useAtom } from "jotai";
import { TransformControls } from "@react-three/drei";

import {
  saveSelectedAtom,
  toolAtom,
  usingGizmoAtom,
} from "../../../helpers/editor/state";

export default function Gizmo() {
  const [tool] = useAtom(toolAtom);
  const [selected, saveSelected] = useAtom(saveSelectedAtom);
  const [, setUsingGizmo] = useAtom(usingGizmoAtom);

  const enabled = Boolean(selected);

  return (
    <TransformControls
      onMouseDown={() => setUsingGizmo(true)}
      onMouseUp={() => {
        saveSelected();
        setUsingGizmo(false);
      }}
      object={selected?.ref?.current}
      showX={enabled}
      showY={enabled}
      showZ={enabled}
      enabled={enabled}
      mode={tool}
    />
  );
}
