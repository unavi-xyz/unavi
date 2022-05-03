import { useStudioStore } from "../../../helpers/studio/store";
import { Tool } from "../../../helpers/studio/types";

import Tooltip from "../../base/Tooltip";

function firstLetterUppercase(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

interface Props {
  tool: Tool;
  children: React.ReactNode;
}

export default function ToolButton({ tool, children }: Props) {
  const selected = useStudioStore((state) => state.tool === tool);
  const selectedClass = selected ? "bg-neutral-100" : "";

  return (
    <Tooltip text={firstLetterUppercase(tool)} placement="bottom">
      <div
        onClick={() => useStudioStore.setState({ tool })}
        className={`h-full aspect-square rounded-lg flex justify-center items-center
                  hover:bg-neutral-100 cursor-pointer text-2xl ${selectedClass}`}
      >
        {children}
      </div>
    </Tooltip>
  );
}
