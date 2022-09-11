import IconButton from "../../../ui/base/IconButton";
import Tooltip from "../../../ui/base/Tooltip";
import { useEditorStore } from "../../store";
import { Tool } from "../../types";

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
  const selected = useEditorStore((state) => state.tool === tool);

  return (
    <div className="h-full aspect-square">
      <Tooltip text={TOOL_TOOLTIPS[tool]} placement="bottom">
        <IconButton
          selected={selected}
          onClick={() => useEditorStore.setState({ tool })}
        >
          {children}
        </IconButton>
      </Tooltip>
    </div>
  );
}
