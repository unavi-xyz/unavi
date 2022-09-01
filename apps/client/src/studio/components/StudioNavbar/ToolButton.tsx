import { useStudioStore } from "../../../studio/store";
import { Tool } from "../../../studio/types";
import IconButton from "../../../ui/base/IconButton";
import Tooltip from "../../../ui/base/Tooltip";

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
    <div className="h-full aspect-square">
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
