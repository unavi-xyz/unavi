"use client";

import { useCallback, useState } from "react";
import { HiCubeTransparent } from "react-icons/hi";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useStudio } from "../Studio";

export default function VisualsButton() {
  const { engine, loaded } = useStudio();

  const [enabled, setEnabled] = useState(false);

  const handleToggleVisuals = useCallback(() => {
    if (!engine || !loaded) return;

    setEnabled((prev) => {
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
