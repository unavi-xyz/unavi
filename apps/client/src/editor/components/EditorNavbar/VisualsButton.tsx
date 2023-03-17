import { HiCubeTransparent } from "react-icons/hi";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useEditorStore } from "../../store";

export default function VisualsButton() {
  const visuals = useEditorStore((state) => state.visuals);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  function handleToggleVisuals() {
    const { engine } = useEditorStore.getState();
    if (!engine || !sceneLoaded) return;

    if (visuals) engine.physics.send({ subject: "stop", data: null });
    else engine.physics.send({ subject: "start", data: null });

    engine.visuals = !visuals;
    useEditorStore.setState({ visuals: !visuals });
  }

  return (
    <Tooltip text={`${visuals ? "Hide" : "Show"} Visuals`} side="bottom">
      <IconButton disabled={!sceneLoaded} selected={visuals} onClick={handleToggleVisuals}>
        <HiCubeTransparent />
      </IconButton>
    </Tooltip>
  );
}
