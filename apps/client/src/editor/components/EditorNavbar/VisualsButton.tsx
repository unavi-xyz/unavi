import { HiCubeTransparent } from "react-icons/hi";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useEditorStore } from "../../store";

export default function VisualsButton() {
  const visuals = useEditorStore((state) => state.visuals);

  function handleToggleVisuals() {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    engine.visuals = !visuals;
    useEditorStore.setState({ visuals: !visuals });
  }

  return (
    <Tooltip text={`${visuals ? "Hide" : "Show"} Visuals`} side="bottom">
      <IconButton selected={visuals} onClick={handleToggleVisuals}>
        <HiCubeTransparent />
      </IconButton>
    </Tooltip>
  );
}
