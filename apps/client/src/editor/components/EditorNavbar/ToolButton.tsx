import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
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
    <div className="aspect-square h-full">
      <Tooltip text={TOOL_TOOLTIPS[tool]} side="bottom">
        <IconButton selected={selected} onClick={() => useEditorStore.setState({ tool })}>
          {children}
        </IconButton>
      </Tooltip>
    </div>
  );
}
