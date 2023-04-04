import { HiCubeTransparent } from "react-icons/hi";

import { useEditorStore } from "@/app/editor/[id]/store";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";

export default function VisualsButton() {
  const showColliders = useEditorStore((state) => state.showColliders);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  function handleToggleVisuals() {
    const { engine } = useEditorStore.getState();
    if (!engine || !sceneLoaded) return;

    if (showColliders) engine.physics.send({ subject: "stop", data: null });
    else engine.physics.send({ subject: "start", data: null });

    engine.showColliders = !showColliders;
    useEditorStore.setState({ showColliders: !showColliders });
  }

  return (
    <Tooltip text={`${showColliders ? "Hide" : "Show"} Colliders`} side="bottom">
      <IconButton disabled={!sceneLoaded} selected={showColliders} onClick={handleToggleVisuals}>
        <HiCubeTransparent />
      </IconButton>
    </Tooltip>
  );
}
