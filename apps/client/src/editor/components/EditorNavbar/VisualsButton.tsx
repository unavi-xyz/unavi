import { useCallback, useState } from "react";
import { HiCubeTransparent } from "react-icons/hi";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useEditor } from "../Editor";

export default function VisualsButton() {
  const { engine, loaded } = useEditor();

  const [enabled, setEnabled] = useState(false);

  const handleToggleVisuals = useCallback(() => {
    if (!engine || !loaded) return;

    setEnabled((prev) => {
      if (prev) engine.physics.send({ subject: "start", data: null });
      else engine.physics.send({ subject: "stop", data: null });

      engine.showColliders = !prev;

      return !prev;
    });
  }, [engine, loaded]);

  return (
    <Tooltip text={`${enabled ? "Hide" : "Show"} Colliders`} side="bottom">
      <IconButton disabled={!loaded} selected={enabled} onClick={handleToggleVisuals}>
        <HiCubeTransparent />
      </IconButton>
    </Tooltip>
  );
}
