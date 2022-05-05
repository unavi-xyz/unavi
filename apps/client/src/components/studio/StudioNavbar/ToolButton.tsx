import { useStudioStore } from "../../../helpers/studio/store";
import { Tool } from "../../../helpers/studio/types";

import Tooltip from "../../base/Tooltip";
import IconButton from "../../base/IconButton";

const TOOL_TOOLTIPS = {
  translate: "Translate (w)",
  rotate: "Rotate (e)",
  scale: "Scale (r)",
};

interface Props {
  tool: Tool;
  children: React.ReactNode;
}

export default function ToolButton({ tool, children }: Props) {
  const selected = useStudioStore((state) => state.tool === tool);

  return (
    <div className="h-full">
      <Tooltip text={TOOL_TOOLTIPS[tool]} placement="bottom">
        <IconButton
          selected={selected}
          onClick={() => useStudioStore.setState({ tool })}
        >
          {children}
        </IconButton>
      </Tooltip>
    </div>
  );
}
